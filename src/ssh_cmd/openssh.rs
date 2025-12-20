use crate::fs::FS;
use crate::instance::TargetInstancePath;
use crate::ssh_cmd::Ssh;
use crate::util::SystemCommand;
use crate::view::{Console, SpinnerView};
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct Openssh {
    known_hosts_file: Option<String>,
    private_keys: Vec<String>,
    args: String,
    cmd: Option<String>,
}

impl Openssh {
    pub fn new() -> Self {
        Self::default()
    }

    fn create_system_command(
        &mut self,
        console: &mut dyn Console,
        user: &str,
        port: u16,
        xforward: bool,
    ) -> SystemCommand {
        let mut command = SystemCommand::new("ssh");

        if let Some(ref known_hosts_file) = self.known_hosts_file {
            Path::new(known_hosts_file)
                .parent()
                .and_then(|dir| dir.to_str())
                .map(|dir| FS::new().create_dir(dir));

            command.arg(format!("-oUserKnownHostsFile={known_hosts_file}"));
        }

        command
            .arg(format!("-p{port}"))
            .arg("-oPreferredAuthentications=publickey,password")
            .arg("-oStrictHostKeyChecking=accept-new")
            .args(
                self.private_keys
                    .iter()
                    .map(|key| format!("-i{key}"))
                    .collect::<Vec<_>>(),
            )
            .args(xforward.then_some("-X").as_slice())
            .args(self.args.split(' ').filter(|item| !item.is_empty()))
            .arg(format!("{}@127.0.0.1", user))
            .args(self.cmd.as_slice());

        console.debug(&command.get_command());
        command
    }

    fn shell_internal(
        &mut self,
        console: &mut dyn Console,
        user: &str,
        port: u16,
        xforward: bool,
    ) -> bool {
        let mut child = self
            .create_system_command(console, user, port, xforward)
            .spawn()
            .unwrap();
        if let Ok(exit) = child.wait() {
            exit.success()
        } else {
            false
        }
    }
}

impl Ssh for Openssh {
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

    fn shell(&mut self, console: &mut dyn Console, user: &str, port: u16, xforward: bool) -> bool {
        loop {
            let start_time = Instant::now();
            if self.shell_internal(console, user, port, xforward) {
                // exit on success
                break;
            }

            if !self.args.is_empty() || start_time.elapsed().as_secs() > 5 {
                // exit if cli command or time expired
                break;
            }

            let spinner = (!console.get_verbosity().is_quiet())
                .then(|| SpinnerView::new("Try to connect".to_string()));
            thread::sleep(Duration::from_secs(5));
            if let Some(mut s) = spinner {
                s.stop()
            }
        }
        true
    }

    fn copy(
        &self,
        console: &mut dyn Console,
        root_dir: &str,
        from: &TargetInstancePath,
        to: &TargetInstancePath,
    ) -> bool {
        let mut command = SystemCommand::new(&format!("{root_dir}/usr/bin/scp"));

        if let Some(ref known_hosts_file) = self.known_hosts_file {
            Path::new(known_hosts_file)
                .parent()
                .and_then(|dir| dir.to_str())
                .map(|dir| FS::new().create_dir(dir));

            command.arg(format!("-oUserKnownHostsFile={known_hosts_file}"));
        }

        command
            .arg("-3")
            .arg("-r")
            .arg(format!("-S{root_dir}/usr/bin/ssh"))
            .args(self.private_keys.iter().map(|key| format!("-i{key}")))
            .args(self.args.split(' ').filter(|item| !item.is_empty()))
            .arg(from.to_scp())
            .arg(to.to_scp());

        console.debug(&command.get_command());
        command
            .set_stdout(!console.get_verbosity().is_quiet())
            .run()
            .is_ok()
    }
}
