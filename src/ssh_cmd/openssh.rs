use crate::fs::FS;
use crate::ssh_cmd::Ssh;
use crate::util::SystemCommand;
use std::path::Path;

#[derive(Default)]
pub struct Openssh {
    known_hosts_file: Option<String>,
    private_keys: Vec<String>,
    user: String,
    port: Option<u16>,
    args: String,
    xforward: bool,
    cmd: Option<String>,
}

impl Openssh {
    pub fn new() -> Self {
        Self::default()
    }

    fn create_system_command(&mut self) -> SystemCommand {
        let mut command = SystemCommand::new("ssh");

        if let Some(ref known_hosts_file) = self.known_hosts_file {
            Path::new(known_hosts_file)
                .parent()
                .and_then(|dir| dir.to_str())
                .map(|dir| FS::new().create_dir(dir));

            command.arg(format!("-oUserKnownHostsFile={known_hosts_file}"));
        }

        command
            .args(self.port.map(|port| format!("-p{port}")).as_slice())
            .arg("-oPreferredAuthentications=publickey,password")
            .arg("-oStrictHostKeyChecking=accept-new")
            .args(
                self.private_keys
                    .iter()
                    .map(|key| format!("-i{key}"))
                    .collect::<Vec<_>>(),
            )
            .args(self.xforward.then_some("-X").as_slice())
            .args(self.args.split(' ').filter(|item| !item.is_empty()))
            .arg(format!("{}@127.0.0.1", self.user))
            .args(self.cmd.as_slice());

        command
    }
}

impl Ssh for Openssh {
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
        let mut child = self.create_system_command().spawn().unwrap();
        if let Ok(exit) = child.wait() {
            exit.success()
        } else {
            false
        }
    }

    fn get_command(&mut self) -> String {
        self.create_system_command().get_command()
    }
}
