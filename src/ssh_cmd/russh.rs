use crate::commands::Context;
use crate::error::Error;
use crate::models::{Instance, TargetInstancePath};
use crate::ssh_cmd::{SftpPath, SshKeyGenerator};
use crate::util;
use crate::view::{Console, Spinner};
use russh::keys::*;
use russh::*;
use russh_sftp::client::SftpSession;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use tokio_util::codec::FramedRead;
use tokio_util::io::StreamReader;

#[derive(PartialEq)]
enum AuthMethod {
    ClientKey,
    Deprecated,
}

/// Polls the terminal geometry every 100ms and propagates changes to
/// the remote PTY. Returns when sending a window change fails.
async fn send_geometry_updates(
    console: &Console<'_>,
    output: &ChannelWriteHalf<client::Msg>,
) -> Result<(), ()> {
    let mut geometry = console.get_geometry();

    loop {
        let new_geometry = console.get_geometry();
        if geometry != new_geometry
            && let Some((width, height)) = new_geometry
        {
            geometry = new_geometry;
            output
                .window_change(width, height, 0, 0)
                .await
                .map_err(|_| ())?;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

pub struct Russh<'a> {
    private_keys: Vec<String>,
    cmd: Option<String>,
    env_vars: Vec<String>,
    context: &'a Context,
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

impl<'a> Russh<'a> {
    pub fn new(context: &'a Context) -> Self {
        Self {
            private_keys: Vec::new(),
            cmd: None,
            env_vars: Vec::new(),
            context,
        }
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
        console: &mut Console<'_>,
        session: &mut russh::client::Handle<Client>,
        user: &str,
        machine: &str,
    ) -> Result<(), ()> {
        loop {
            let password =
                console.prompt_secret(&format!("Enter password for {user}@{machine}: "))?;

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
        console: &mut Console<'_>,
        session: &mut russh::client::Handle<Client>,
        user: &str,
        machine: &str,
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
        self.authenticate_with_password(console, session, user, machine)
            .await
            .map(|_| AuthMethod::Deprecated)
    }

    fn warn_deprecated_auth(
        &self,
        console: &mut Console<'_>,
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
        console: &mut Console<'_>,
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

        console.debug(&format!("Connected to 127.0.0.1:{port}"));
        console.play(Arc::new(std::sync::Mutex::new(Spinner::new(format!(
            "Authenticating on {machine}"
        )))));

        let auth_method = self
            .authenticate(console, &mut session, user, machine, client_key)
            .await;
        console.stop();

        if auth_method? == AuthMethod::Deprecated {
            self.warn_deprecated_auth(console, machine, client_key).ok();
        }

        session.channel_open_session().await.map_err(|_| ())
    }

    async fn handle_interactive_shell(
        &self,
        console: &mut Console<'_>,
        machine: &str,
        client_key: &str,
        user: &str,
        port: u16,
    ) -> Result<(), ()> {
        let channel = self
            .open_channel(console, machine, client_key, user, port)
            .await?;
        let (w, h) = console.get_geometry().unwrap();

        console.play(Arc::new(std::sync::Mutex::new(Spinner::new(format!(
            "Opening shell on {machine}"
        )))));
        channel
            .request_pty(
                false,
                &self
                    .context
                    .get_system()
                    .read_env_var("TERM")
                    .unwrap_or_else(|| "xterm".into()),
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
                (
                    var.clone(),
                    self.context
                        .get_system()
                        .read_env_var(var)
                        .unwrap_or_default(),
                )
            };
            channel.set_env(false, name, value).await.map_err(|_| ())?;
        }

        if let Some(cmd) = &self.cmd {
            channel.exec(true, cmd.as_str()).await.map_err(|_| ())?;
        } else {
            channel.request_shell(true).await.map_err(|_| ())?;
        }
        let (mut ssh_in, ssh_out) = channel.split();
        let mut ssh_reader = ssh_in.make_reader();
        let mut ssh_writer = ssh_out.make_writer();

        console.stop();
        console.raw_mode();
        let mut stdin = StreamReader::new(FramedRead::new(
            tokio::io::stdin(),
            util::ShortcutDecoder::new(),
        ));
        let mut stdout = tokio::io::stdout();
        tokio::select!(
            _ = tokio::io::copy(&mut stdin, &mut ssh_writer) => {},
            _ = tokio::io::copy(&mut ssh_reader, &mut stdout) => {},
            _ = send_geometry_updates(console, &ssh_out) => {},
        );
        console.reset();
        Ok(())
    }

    async fn open_sftp(
        &self,
        console: &mut Console<'_>,
        instance: &Instance,
        user: &Option<String>,
        client_key: &str,
    ) -> Rc<SftpSession> {
        let user = user.as_deref().unwrap_or(instance.user.as_str());
        let channel = self
            .open_channel(console, &instance.name, client_key, user, instance.ssh_port)
            .await
            .unwrap();
        channel.request_subsystem(true, "sftp").await.unwrap();
        Rc::new(SftpSession::new(channel.into_stream()).await.unwrap())
    }

    async fn open_target_fs(
        &self,
        console: &mut Console<'_>,
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
        console: &mut Console<'_>,
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
        console: &mut Console<'_>,
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
        console: &mut Console<'_>,
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
