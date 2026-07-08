use crate::error::Error;
use crate::models::{Instance, TargetInstancePath};
use crate::ssh_cmd::{SftpPath, SshKeyGenerator};
use crate::util;
use crate::view::{Console, Spinner};
use russh::keys::*;
use russh::*;
use russh_sftp::client::SftpSession;
use std::env;
use std::io::{Cursor, Write};
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use tokio::{io::AsyncReadExt, sync::Mutex};

#[derive(PartialEq)]
enum AuthMethod {
    ClientKey,
    Deprecated,
}

async fn read_password(console: &mut dyn Console, user: &str) -> Result<String, ()> {
    let mut stdout = std::io::stdout();
    let mut stdin = tokio::io::stdin();

    let mut password = Vec::new();
    let c: &mut [u8] = &mut [0];

    stdout
        .write_all(format!("{user}@localhost's password: ").as_bytes())
        .map_err(|_| ())?;

    stdout.flush().map_err(|_| ())?;

    console.raw_mode();
    loop {
        // read character from stdin
        if stdin.read(c).await.map_err(|_| ())? == 0 {
            continue;
        }

        match c[0] {
            // Ctrl+C
            0x03 => {
                console.reset();
                println!();
                std::process::exit(1)
            }

            // Carriage return and line feed
            0x0A | 0x0D => break,

            // Delete and backspace
            0x7E | 0x7F => {
                password.pop();
            }

            byte => {
                password.push(byte);
            }
        };
    }
    console.reset();

    print!("\r\n");
    str::from_utf8(&password)
        .map(|password| password.to_string())
        .map_err(|_| ())
}

async fn ssh_geometry(
    console: &mut dyn Console,
    output: Arc<Mutex<ChannelWriteHalf<client::Msg>>>,
) -> Result<(), ()> {
    let mut geometry = console.get_geometry();

    loop {
        // update terminal geometry
        let new_geometry = console.get_geometry();
        if geometry != new_geometry
            && let Some(new_geometry) = new_geometry
        {
            geometry = Some(new_geometry);
            output
                .lock()
                .await
                .window_change(new_geometry.0, new_geometry.1, 0, 0)
                .await
                .map_err(|_| ())?;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

async fn ssh_output(output: Arc<Mutex<ChannelWriteHalf<client::Msg>>>) -> Result<(), ()> {
    let mut stdin = tokio::io::stdin();
    let mut c: &mut [u8] = &mut [0];

    loop {
        // read character from stdin
        if stdin.read(c).await.map_err(|_| ())? == 0 {
            continue;
        }

        // send character to ssh server
        output
            .lock()
            .await
            .data(&mut Cursor::new(&mut c))
            .await
            .map_err(|_| ())?;
    }
}

#[derive(Default)]
pub struct Russh {
    private_keys: Vec<String>,
    cmd: Option<String>,
    env_vars: Vec<String>,
}

struct Client {}

impl client::Handler for Client {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

impl Russh {
    pub fn new() -> Self {
        Self::default()
    }

    async fn authenticate_with_default_password(
        &self,
        session: &mut russh::client::Handle<Client>,
        user: &str,
    ) -> Result<(), ()> {
        let auth = session
            .authenticate_password(user, "cubic")
            .await
            .map(|auth| auth.success());

        if let Ok(true) = auth { Ok(()) } else { Err(()) }
    }

    async fn authenticate_with_keys(
        &self,
        session: &mut russh::client::Handle<Client>,
        user: &str,
        keys: &[String],
    ) -> bool {
        let Ok(hash_alg) = session.best_supported_rsa_hash().await else {
            return false;
        };
        let hash_alg = hash_alg.flatten();

        for key in keys {
            if let Ok(key_pair) = load_secret_key(key, None)
                && let Ok(auth) = session
                    .authenticate_publickey(
                        user,
                        PrivateKeyWithHashAlg::new(Arc::new(key_pair), hash_alg),
                    )
                    .await
                && auth.success()
            {
                return true;
            }
        }

        false
    }

    async fn authenticate_with_password(
        &self,
        console: &mut dyn Console,
        session: &mut russh::client::Handle<Client>,
        user: &str,
    ) -> Result<(), ()> {
        loop {
            let password = read_password(console, user).await?;

            if session
                .authenticate_password(user, password)
                .await
                .map_err(|_| ())?
                .success()
            {
                break;
            }
        }

        Ok(())
    }

    async fn authenticate(
        &self,
        console: &mut dyn Console,
        session: &mut russh::client::Handle<Client>,
        user: &str,
        client_key: &str,
    ) -> Result<AuthMethod, ()> {
        // The cubic per-instance ssh_client_key is the only supported method.
        // Everything below is a deprecated fallback.
        console.debug(&format!(
            "Authenticating '{user}' with client key '{client_key}'"
        ));
        if self
            .authenticate_with_keys(session, user, &[client_key.to_string()])
            .await
        {
            console.debug("Authenticated with client key");
            return Ok(AuthMethod::ClientKey);
        }

        console.debug(&format!(
            "Client key failed, trying {} deprecated key(s)",
            self.private_keys.len()
        ));
        if self
            .authenticate_with_keys(session, user, &self.private_keys)
            .await
        {
            console.debug("Authenticated with a deprecated private key");
            return Ok(AuthMethod::Deprecated);
        }

        console.debug("Deprecated keys failed, trying the default password");
        if self
            .authenticate_with_default_password(session, user)
            .await
            .is_ok()
        {
            console.debug("Authenticated with the default password");
            return Ok(AuthMethod::Deprecated);
        }

        console.debug("Default password failed, prompting for a password");
        self.authenticate_with_password(console, session, user)
            .await
            .map(|_| AuthMethod::Deprecated)
    }

    fn warn_deprecated_auth(
        &self,
        console: &mut dyn Console,
        machine: &str,
        client_key: &str,
    ) -> Result<(), ()> {
        // create the cubic ssh key if it does not exist yet
        if !Path::new(client_key).exists() {
            SshKeyGenerator::new()
                .generate_key(Path::new(client_key))
                .map_err(|_| ())?;
        }

        let pubkey = SshKeyGenerator::new()
            .generate_public_key(Path::new(client_key))
            .map_err(|_| ())?;

        console.warn(&format!(
            "Connected to '{machine}' using a deprecated authentication method."
        ));
        console.warn(&format!(
            "Add the following cubic SSH key on '{machine}' to ~/.ssh/authorized_keys:"
        ));
        console.print("");
        console.print(&pubkey);
        console.print("");

        Ok(())
    }

    async fn open_channel(
        &self,
        console: &mut dyn Console,
        machine: &str,
        client_key: &str,
        user: &str,
        port: u16,
    ) -> Result<Channel<russh::client::Msg>, ()> {
        let mut session;

        console.play(Arc::new(std::sync::Mutex::new(Spinner::new(format!(
            "Connecting to {machine}"
        )))));
        console.debug(&format!("Connecting to 127.0.0.1:{port}"));
        let mut failed = false;
        loop {
            let sh = Client {};
            let addrs = ("127.0.0.1", port);
            let config = Arc::new(client::Config::default());
            if let Ok(s) = client::connect(config, addrs, sh).await.map_err(|_| ()) {
                session = s;
                break;
            }
            if !failed {
                failed = true;
                console.debug(&format!("Connection to 127.0.0.1:{port} failed, retrying"));
            }
        }

        console.stop();
        console.debug(&format!("Connected to 127.0.0.1:{port}"));

        if self
            .authenticate(console, &mut session, user, client_key)
            .await?
            == AuthMethod::Deprecated
        {
            self.warn_deprecated_auth(console, machine, client_key).ok();
        }

        session.channel_open_session().await.map_err(|_| ())
    }

    async fn handle_interactive_shell(
        &self,
        console: &mut dyn Console,
        machine: &str,
        client_key: &str,
        user: &str,
        port: u16,
    ) -> Result<(), ()> {
        let channel = self
            .open_channel(console, machine, client_key, user, port)
            .await?;
        let (w, h) = console.get_geometry().unwrap();
        channel
            .request_pty(
                false,
                &env::var("TERM").unwrap_or("xterm".into()),
                w,
                h,
                0,
                0,
                &[],
            )
            .await
            .map_err(|_| ())?;

        for var in &self.env_vars {
            let (name, value) = if let Some((k, v)) = var.split_once('=') {
                (k.to_string(), v.to_string())
            } else {
                (var.clone(), env::var(var).unwrap_or_default())
            };
            channel.set_env(false, name, value).await.map_err(|_| ())?;
        }

        if let Some(cmd) = &self.cmd {
            channel.exec(true, cmd.as_str()).await.map_err(|_| ())?;
        } else {
            channel.request_shell(true).await.map_err(|_| ())?;
        }
        let (mut ssh_in, ssh_out) = channel.split();
        let mut ssh_in = ssh_in.make_reader();
        let mut stdout = tokio::io::stdout();

        let ssh_out = Arc::new(Mutex::new(ssh_out));

        console.raw_mode();
        tokio::select!(
            _ = ssh_geometry(console, ssh_out.clone()) => { console.reset(); std::process::exit(0); },
            _ = ssh_output(ssh_out.clone()) => { console.reset(); std::process::exit(0); },
            _ = tokio::io::copy(&mut ssh_in, &mut stdout) => { console.reset(); std::process::exit(0); },
            else => {}
        );
        Ok(())
    }

    async fn open_sftp(
        &self,
        console: &mut dyn Console,
        instance: &Instance,
        user: &Option<String>,
        client_key: &str,
    ) -> Rc<SftpSession> {
        let user = user.as_ref().unwrap_or(&instance.user);
        let channel = self
            .open_channel(console, &instance.name, client_key, user, instance.ssh_port)
            .await
            .unwrap();
        channel.request_subsystem(true, "sftp").await.unwrap();
        Rc::new(SftpSession::new(channel.into_stream()).await.unwrap())
    }

    async fn open_target_fs(
        &self,
        console: &mut dyn Console,
        path: &TargetInstancePath,
        client_key: Option<&str>,
    ) -> SftpPath {
        let sftp = if let Some(instance) = &path.instance {
            Some(
                self.open_sftp(
                    console,
                    instance,
                    &path.user,
                    client_key.unwrap_or_default(),
                )
                .await,
            )
        } else {
            None
        };
        SftpPath {
            sftp,
            path: path.to_pathbuf(),
        }
    }

    async fn async_copy(
        &self,
        console: &mut dyn Console,
        _root_dir: &str,
        from: &TargetInstancePath,
        from_key: Option<&str>,
        to: &TargetInstancePath,
        to_key: Option<&str>,
    ) -> Result<(), Error> {
        let source = self.open_target_fs(console, from, from_key).await;
        let target = self.open_target_fs(console, to, to_key).await;

        source.copy(console, target).await
    }

    pub fn set_private_keys(&mut self, private_keys: Vec<String>) {
        self.private_keys = private_keys;
    }

    pub fn set_cmd(&mut self, cmd: Option<String>) {
        self.cmd = cmd;
    }

    pub fn set_env_vars(&mut self, env_vars: Vec<String>) {
        self.env_vars = env_vars;
    }

    pub fn shell(
        &mut self,
        console: &mut dyn Console,
        machine: &str,
        client_key: &str,
        user: &str,
        port: u16,
    ) -> bool {
        util::AsyncCaller::new()
            .call(self.handle_interactive_shell(console, machine, client_key, user, port))
            .is_ok()
    }

    pub fn copy(
        &self,
        console: &mut dyn Console,
        root_dir: &str,
        from: &TargetInstancePath,
        from_key: Option<&str>,
        to: &TargetInstancePath,
        to_key: Option<&str>,
    ) -> Result<(), Error> {
        util::AsyncCaller::new()
            .call(self.async_copy(console, root_dir, from, from_key, to, to_key))
    }
}
