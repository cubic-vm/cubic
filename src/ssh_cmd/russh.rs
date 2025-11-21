use crate::ssh_cmd::Ssh;
use crate::util::terminal;
use crate::view::Console;
use russh::keys::*;
use russh::*;
use std::env;
use std::io::{Read, Write};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;

#[derive(Default)]
pub struct Russh {
    known_hosts_file: Option<String>,
    private_keys: Vec<String>,
    args: String,
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

async fn handle_stdin_stream<S: From<(ChannelId, ChannelMsg)> + Send + Sync + 'static>(
    out: ChannelWriteHalf<S>,
) -> Result<(), ()> {
    let stdin = std::io::stdin();

    for byte in stdin.bytes() {
        let buffer = [byte.unwrap()];
        out.data(std::io::Cursor::new(buffer))
            .await
            .map_err(|_| ())?;
        tokio::task::yield_now().await;
    }

    Ok(())
}

async fn handle_stdout_stream(mut ssh_in: ChannelReadHalf) -> Result<(), ()> {
    let mut stdout = tokio::io::stdout();

    loop {
        if let Some(msg) = ssh_in.wait().await {
            match msg {
                ChannelMsg::Data { ref data } => {
                    stdout.write_all(data).await.map_err(|_| ())?;
                    stdout.flush().await.map_err(|_| ())?;
                }
                ChannelMsg::ExitStatus { .. } => {
                    break;
                }
                _ => {}
            }
        } else {
            break;
        }

        tokio::task::yield_now().await;
    }

    Ok(())
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
            if let Ok(key_pair) = load_secret_key(key, None) {
                if let Ok(auth) = session
                    .authenticate_publickey(
                        user,
                        PrivateKeyWithHashAlg::new(Arc::new(key_pair), hash_alg),
                    )
                    .await
                {
                    if auth.success() {
                        return Ok(());
                    }
                }
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

    async fn open_channel(&self, user: &str, port: u16) -> Result<Channel<russh::client::Msg>, ()> {
        let config = client::Config {
            inactivity_timeout: Some(Duration::from_secs(60)),
            ..<_>::default()
        };

        let config = Arc::new(config);
        let sh = Client {};
        let addrs = ("127.0.0.1", port);

        let mut session = client::connect(config, addrs, sh).await.map_err(|_| ())?;

        self.authenticate(&mut session, user).await?;

        session.channel_open_session().await.map_err(|_| ())
    }

    async fn handle_interactive_shell(&self, user: &str, port: u16) -> Result<(), ()> {
        let channel = self.open_channel(user, port).await?;
        let (w, h) = termion::terminal_size().map_err(|_| ())?;
        channel
            .request_pty(
                false,
                &env::var("TERM").unwrap_or("xterm".into()),
                w as u32,
                h as u32,
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
        let (ssh_in, ssh_out) = channel.split();

        let stdin_stream = tokio::spawn(handle_stdin_stream(ssh_out));
        handle_stdout_stream(ssh_in).await?;
        stdin_stream.abort();

        Ok(())
    }
}

impl Ssh for Russh {
    fn set_known_hosts_file(&mut self, path: Option<String>) {
        self.known_hosts_file = path;
    }

    fn set_private_keys(&mut self, private_keys: Vec<String>) {
        self.private_keys = private_keys;
    }

    fn set_args(&mut self, args: String) {
        self.args = args;
    }

    fn set_cmd(&mut self, cmd: Option<String>) {
        self.cmd = cmd;
    }

    fn shell(
        &mut self,
        _console: &mut dyn Console,
        user: &str,
        port: u16,
        _xforward: bool,
    ) -> bool {
        let termios_original = terminal::term_raw_mode();

        let result = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(self.handle_interactive_shell(user, port))
            .is_ok();

        terminal::term_reset(termios_original);
        result
    }

    fn copy(&self, _console: &mut dyn Console, _root_dir: &str, _from: &str, _to: &str) -> bool {
        false
    }
}
