use crate::ssh_cmd::Ssh;
use crate::util::terminal;
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
    user: String,
    port: Option<u16>,
    args: String,
    xforward: bool,
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
    ) -> Result<(), ()> {
        let auth = session
            .authenticate_password(&self.user, "cubic")
            .await
            .map(|auth| auth.success());

        if let Ok(true) = auth { Ok(()) } else { Err(()) }
    }

    async fn authenticate_with_password(
        &self,
        session: &mut russh::client::Handle<Client>,
    ) -> Result<(), ()> {
        let user = &self.user;
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

    async fn authenticate(&self, session: &mut russh::client::Handle<Client>) -> Result<(), ()> {
        if self
            .authenticate_with_default_password(session)
            .await
            .is_ok()
        {
            return Ok(());
        }

        self.authenticate_with_password(session).await
    }

    async fn open_channel(&self) -> Result<Channel<russh::client::Msg>, ()> {
        let port = self.port.ok_or(())?;

        let config = client::Config {
            inactivity_timeout: Some(Duration::from_secs(60)),
            ..<_>::default()
        };

        let config = Arc::new(config);
        let sh = Client {};
        let addrs = ("127.0.0.1", port);

        let mut session = client::connect(config, addrs, sh).await.map_err(|_| ())?;

        self.authenticate(&mut session).await?;

        session.channel_open_session().await.map_err(|_| ())
    }

    async fn handle_interactive_shell(&self) -> Result<(), ()> {
        let channel = self.open_channel().await?;
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

        channel.request_shell(true).await.map_err(|_| ())?;
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

    fn set_user(&mut self, user: String) {
        self.user = user;
    }

    fn set_port(&mut self, port: Option<u16>) {
        self.port = port;
    }

    fn set_args(&mut self, args: String) {
        self.args = args;
    }

    fn set_xforward(&mut self, xforward: bool) {
        self.xforward = xforward;
    }

    fn set_cmd(&mut self, cmd: Option<String>) {
        self.cmd = cmd;
    }

    fn connect(&mut self) -> bool {
        let termios_original = terminal::term_raw_mode();

        let result = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(self.handle_interactive_shell())
            .is_ok();

        terminal::term_reset(termios_original);
        result
    }

    fn get_command(&mut self) -> String {
        String::new()
    }
}
