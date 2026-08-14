use crate::commands::Context;
use crate::error::Error;
use crate::models::{Instance, TargetInstancePath};
use crate::ssh::{HostKeyChecker, KeyCheck, SftpPath, SshKeyGenerator};
use crate::util;
use crate::view::{ConfirmDialog, Console, Spinner};
use russh::keys::*;
use russh::*;
use russh_sftp::client::SftpSession;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
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

pub struct SshClient<'a> {
    private_keys: Vec<String>,
    cmd: Option<String>,
    env_vars: Vec<String>,
    context: &'a Context,
}

/// Verifies the host key of the guest. The handler is moved into the russh
/// session, so it cannot borrow the console or the context. It records the
/// offered key instead and lets the caller report and store it.
struct ServerKeyHandler {
    pinned: Option<String>,
    offered: Arc<Mutex<Option<String>>>,
}

impl client::Handler for ServerKeyHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        let Ok(key) = server_public_key.to_openssh() else {
            return Ok(false);
        };
        let check = HostKeyChecker::new().check_key(self.pinned.as_deref(), &key);
        *self.offered.lock().unwrap() = Some(key);

        Ok(check != KeyCheck::Changed)
    }
}

impl<'a> SshClient<'a> {
    pub fn new(context: &'a Context) -> Self {
        Self {
            private_keys: Vec::new(),
            cmd: None,
            env_vars: Vec::new(),
            context,
        }
    }

    async fn authenticate_with_keys(
        &self,
        session: &mut russh::client::Handle<ServerKeyHandler>,
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
        session: &mut russh::client::Handle<ServerKeyHandler>,
        user: &str,
        machine: &str,
    ) -> Result<(), Error> {
        loop {
            let password = console
                .prompt_secret(&format!("Enter password for {user}@{machine}: "))
                .map_err(|_| Error::SshAuthCancelled(machine.to_string()))?;

            if session
                .authenticate_password(user, password)
                .await
                .map_err(|_| Error::SshAuthFailed(machine.to_string()))?
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
        session: &mut russh::client::Handle<ServerKeyHandler>,
        user: &str,
        machine: &str,
        client_key: &str,
    ) -> Result<AuthMethod, Error> {
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

        console.debug("Deprecated keys failed, prompting for a password");
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
        if !self.context.get_system().exists_path(Path::new(client_key)) {
            SshKeyGenerator::new()
                .generate_key(self.context.get_system(), Path::new(client_key))
                .map_err(|_| ())?;
        }

        let pubkey = SshKeyGenerator::new()
            .generate_public_key(self.context.get_system(), Path::new(client_key))
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

    /// Reports a host key that does not match the pinned one and asks whether
    /// to trust it from now on.
    fn confirm_new_host_key(
        &self,
        console: &mut Console<'_>,
        machine: &str,
        pinned: &str,
        offered: &str,
    ) -> bool {
        let checker = HostKeyChecker::new();

        console.stop();
        console.warn(&format!(
            "The host key of instance '{machine}' does not match the stored key."
        ));
        console.warn(&format!("  expected  {}", checker.get_fingerprint(pinned)));
        console.warn(&format!("  actual    {}", checker.get_fingerprint(offered)));
        console.warn("This may be a malicious attempt to take over the connection to the guest.");

        ConfirmDialog::new("Do you want to trust the new key and continue?").confirm(console)
    }

    async fn open_channel(
        &self,
        console: &mut Console<'_>,
        machine: &str,
        client_key: &str,
        user: &str,
        port: u16,
    ) -> Result<Channel<russh::client::Msg>, Error> {
        let mut session;
        let store = self.context.get_instance_store();
        let mut instance = store.load(machine)?;
        let mut pinned = instance.ssh_host_key.clone();
        let offered = Arc::new(Mutex::new(None));

        console.play(Arc::new(Mutex::new(Spinner::new(format!(
            "Connecting to {machine}"
        )))));
        console.debug(&format!("Connecting to 127.0.0.1:{port}"));
        let mut failed = false;
        loop {
            let sh = ServerKeyHandler {
                pinned: pinned.clone(),
                offered: Arc::clone(&offered),
            };
            let addrs = ("127.0.0.1", port);
            let config = Arc::new(client::Config::default());
            if let Ok(s) = client::connect(config, addrs, sh).await {
                session = s;
                break;
            }

            // A rejected host key fails like any other connect error, so it has
            // to end the retry loop on its own.
            let key = offered.lock().unwrap().clone();
            if let (Some(key), Some(pinned_key)) = (key, pinned.clone())
                && HostKeyChecker::new().check_key(Some(&pinned_key), &key) == KeyCheck::Changed
            {
                if !self.confirm_new_host_key(console, machine, &pinned_key, &key) {
                    return Err(Error::SshHostKeyRejected(machine.to_string()));
                }

                instance.ssh_host_key = Some(key.clone());
                store.store(&instance)?;
                pinned = Some(key);
                console.play(Arc::new(Mutex::new(Spinner::new(format!(
                    "Connecting to {machine}"
                )))));
                continue;
            }

            if !failed {
                failed = true;
                console.debug(&format!("Connection to 127.0.0.1:{port} failed, retrying"));
            }
        }

        console.debug(&format!("Connected to 127.0.0.1:{port}"));

        // Trust the key of a guest that has none stored yet on this first connect.
        if pinned.is_none() {
            let key = offered.lock().unwrap().clone();
            if let Some(key) = key {
                console.debug(&format!(
                    "Pinning host key {}",
                    HostKeyChecker::new().get_fingerprint(&key)
                ));
                instance.ssh_host_key = Some(key);
                store.store(&instance)?;
            }
        }

        console.play(Arc::new(Mutex::new(Spinner::new(format!(
            "Authenticating on {machine}"
        )))));

        let auth_method = self
            .authenticate(console, &mut session, user, machine, client_key)
            .await;
        console.stop();

        if auth_method? == AuthMethod::Deprecated {
            self.warn_deprecated_auth(console, machine, client_key).ok();
        }

        session
            .channel_open_session()
            .await
            .map_err(|_| Error::SshConnectionFailed(machine.to_string()))
    }

    async fn handle_interactive_shell(
        &self,
        console: &mut Console<'_>,
        machine: &str,
        client_key: &str,
        user: &str,
        port: u16,
    ) -> Result<(), Error> {
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
            .map_err(|_| Error::SshConnectionFailed(machine.to_string()))?;

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
            channel
                .set_env(false, name, value)
                .await
                .map_err(|_| Error::SshConnectionFailed(machine.to_string()))?;
        }

        if let Some(cmd) = &self.cmd {
            channel
                .exec(true, cmd.as_str())
                .await
                .map_err(|_| Error::SshConnectionFailed(machine.to_string()))?;
        } else {
            channel
                .request_shell(true)
                .await
                .map_err(|_| Error::SshConnectionFailed(machine.to_string()))?;
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
    ) -> Result<Rc<SftpSession>, Error> {
        let user = user.as_deref().unwrap_or(instance.user.as_str());
        let channel = self
            .open_channel(console, &instance.name, client_key, user, instance.ssh_port)
            .await?;
        channel
            .request_subsystem(true, "sftp")
            .await
            .map_err(|error| Error::Sftp(error.to_string()))?;
        SftpSession::new(channel.into_stream())
            .await
            .map(Rc::new)
            .map_err(|error| Error::Sftp(error.to_string()))
    }

    async fn open_target_fs(
        &self,
        console: &mut Console<'_>,
        path: &TargetInstancePath,
        client_key: Option<&str>,
    ) -> Result<SftpPath, Error> {
        let sftp = if let Some(instance) = &path.instance {
            Some(
                self.open_sftp(
                    console,
                    instance,
                    &path.user,
                    client_key.unwrap_or_default(),
                )
                .await?,
            )
        } else {
            None
        };
        Ok(SftpPath {
            sftp,
            path: path.to_pathbuf(),
        })
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
        let source = self.open_target_fs(console, from, from_key).await?;
        let target = self.open_target_fs(console, to, to_key).await?;

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
    ) -> Result<(), Error> {
        util::AsyncCaller::new()
            .call(self.handle_interactive_shell(console, machine, client_key, user, port))
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

#[cfg(test)]
mod tests {
    use super::*;
    use getrandom::SysRng;
    use getrandom::rand_core::UnwrapErr;
    use russh::client::Handler;
    use russh::keys::ssh_key::{Algorithm, PrivateKey};

    fn build_key() -> ssh_key::PublicKey {
        PrivateKey::random(&mut UnwrapErr(SysRng), Algorithm::Ed25519)
            .unwrap()
            .public_key()
            .clone()
    }

    fn check_key(
        pinned: Option<&ssh_key::PublicKey>,
        offered: &ssh_key::PublicKey,
    ) -> (bool, bool) {
        let seen = Arc::new(Mutex::new(None));
        let mut handler = ServerKeyHandler {
            pinned: pinned.map(|key| key.to_openssh().unwrap()),
            offered: Arc::clone(&seen),
        };

        let accepted = util::AsyncCaller::new()
            .call(handler.check_server_key(offered))
            .unwrap();
        let recorded =
            seen.lock().unwrap().as_deref() == Some(offered.to_openssh().unwrap().as_str());

        (accepted, recorded)
    }

    #[test]
    fn test_check_server_key_accepts_the_first_key() {
        assert_eq!(check_key(None, &build_key()), (true, true));
    }

    #[test]
    fn test_check_server_key_accepts_the_pinned_key() {
        let key = build_key();

        assert_eq!(check_key(Some(&key), &key), (true, true));
    }

    #[test]
    fn test_check_server_key_rejects_a_changed_key() {
        assert_eq!(check_key(Some(&build_key()), &build_key()), (false, true));
    }
}
