use crate::instance::{Instance, TargetInstancePath};
use crate::ssh_cmd::SftpPath;
use crate::util;
use crate::view::{Console, SpinnerView};
use russh::keys::*;
use russh::*;
use russh_sftp::client::SftpSession;
use std::env;
use std::io::{Cursor, Write};
use std::rc::Rc;
use std::sync::Arc;
use tokio::{io::AsyncReadExt, sync::Mutex};

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

    pub async fn is_running(&self, port: u16) -> bool {
        client::connect(
            Arc::new(client::Config::default()),
            ("127.0.0.1", port),
            Client {},
        )
        .await
        .is_ok()
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

    async fn authenticate_with_pubkey(
        &self,
        session: &mut russh::client::Handle<Client>,
        user: &str,
    ) -> Result<(), ()> {
        let hash_alg = session
            .best_supported_rsa_hash()
            .await
            .map_err(|_| ())?
            .flatten();

        for key in &self.private_keys {
            if let Ok(key_pair) = load_secret_key(key, None)
                && let Ok(auth) = session
                    .authenticate_publickey(
                        user,
                        PrivateKeyWithHashAlg::new(Arc::new(key_pair), hash_alg),
                    )
                    .await
                && auth.success()
            {
                return Ok(());
            }
        }

        Err(())
    }

    async fn authenticate_with_password(
        &self,
        session: &mut russh::client::Handle<Client>,
        user: &str,
    ) -> Result<(), ()> {
        loop {
            let mut stdout = std::io::stdout();
            stdout
                .write_all(format!("{user}@localhost's password: ").as_bytes())
                .map_err(|_| ())?;
            stdout.flush().map_err(|_| ())?;
            let mut password = String::new();
            std::io::stdin().read_line(&mut password).map_err(|_| ())?;
            println!();
            let auth_res = session
                .authenticate_password(user, &password[..password.len() - 1])
                .await
                .map_err(|_| ())?;

            if auth_res.success() {
                break;
            }
        }

        Ok(())
    }

    async fn authenticate(
        &self,
        session: &mut russh::client::Handle<Client>,
        user: &str,
    ) -> Result<(), ()> {
        if self
            .authenticate_with_default_password(session, user)
            .await
            .is_ok()
        {
            return Ok(());
        }

        if self.authenticate_with_pubkey(session, user).await.is_ok() {
            return Ok(());
        }

        self.authenticate_with_password(session, user).await
    }

    async fn open_channel(
        &self,
        console: &mut dyn Console,
        user: &str,
        port: u16,
    ) -> Result<Channel<russh::client::Msg>, ()> {
        let mut session;

        let spinner = (!console.get_verbosity().is_quiet())
            .then(|| SpinnerView::new("Connecting".to_string()));
        loop {
            let sh = Client {};
            let addrs = ("127.0.0.1", port);
            let config = Arc::new(client::Config::default());
            if let Ok(s) = client::connect(config, addrs, sh).await.map_err(|_| ()) {
                session = s;
                break;
            }
        }

        self.authenticate(&mut session, user).await?;

        if let Some(mut s) = spinner {
            s.stop()
        }

        session.channel_open_session().await.map_err(|_| ())
    }

    async fn handle_interactive_shell(
        &self,
        console: &mut dyn Console,
        user: &str,
        port: u16,
    ) -> Result<(), ()> {
        let channel = self.open_channel(console, user, port).await?;
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

        if let Some(cmd) = &self.cmd {
            channel.exec(true, cmd.as_str()).await.map_err(|_| ())?;
        } else {
            channel.request_shell(true).await.map_err(|_| ())?;
        }
        let (mut ssh_in, ssh_out) = channel.split();
        let mut ssh_in = ssh_in.make_reader();
        let mut stdout = tokio::io::stdout();

        let ssh_out = Arc::new(Mutex::new(ssh_out));

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
    ) -> Rc<SftpSession> {
        let user = user.as_ref().unwrap_or(&instance.user);
        let channel = self
            .open_channel(console, user, instance.ssh_port)
            .await
            .unwrap();
        channel.request_subsystem(true, "sftp").await.unwrap();
        Rc::new(SftpSession::new(channel.into_stream()).await.unwrap())
    }

    async fn open_target_fs(
        &self,
        console: &mut dyn Console,
        path: &TargetInstancePath,
    ) -> SftpPath {
        let sftp = if let Some(instance) = &path.instance {
            Some(self.open_sftp(console, instance, &path.user).await)
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
        to: &TargetInstancePath,
    ) -> Result<(), ()> {
        let source = self.open_target_fs(console, from).await;
        let target = self.open_target_fs(console, to).await;

        source.copy(target).await;
        Ok(())
    }

    pub fn set_private_keys(&mut self, private_keys: Vec<String>) {
        self.private_keys = private_keys;
    }

    pub fn set_cmd(&mut self, cmd: Option<String>) {
        self.cmd = cmd;
    }

    pub fn shell(&mut self, console: &mut dyn Console, user: &str, port: u16) -> bool {
        console.raw_mode();
        let result = util::AsyncCaller::new()
            .call(self.handle_interactive_shell(console, user, port))
            .is_ok();
        console.reset();
        result
    }

    pub fn copy(
        &self,
        console: &mut dyn Console,
        root_dir: &str,
        from: &TargetInstancePath,
        to: &TargetInstancePath,
    ) -> bool {
        util::AsyncCaller::new()
            .call(self.async_copy(console, root_dir, from, to))
            .is_ok()
    }
}
