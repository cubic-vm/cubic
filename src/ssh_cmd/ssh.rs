use crate::fs::FS;
use crate::util;
use std::path::Path;
use std::process::Command;

#[derive(Default)]
pub struct Ssh {
    known_hosts_file: Option<String>,
    private_keys: Vec<String>,
    user: String,
    port: Option<u16>,
    args: String,
    verbose: bool,
    xforward: bool,
    cmd: Option<String>,
}

impl Ssh {
    pub fn new() -> Self {
        Ssh::default()
    }

    pub fn set_known_hosts_file(&mut self, path: Option<String>) -> &mut Self {
        self.known_hosts_file = path;
        self
    }

    pub fn set_private_keys(&mut self, private_keys: Vec<String>) -> &mut Self {
        self.private_keys = private_keys;
        self
    }

    pub fn set_user(&mut self, user: String) -> &mut Self {
        self.user = user;
        self
    }

    pub fn set_port(&mut self, port: Option<u16>) -> &mut Self {
        self.port = port;
        self
    }

    pub fn set_args(&mut self, args: String) -> &mut Self {
        self.args = args;
        self
    }

    pub fn set_verbose(&mut self, verbose: bool) -> &mut Self {
        self.verbose = verbose;
        self
    }

    pub fn set_xforward(&mut self, xforward: bool) -> &mut Self {
        self.xforward = xforward;
        self
    }

    pub fn set_cmd(&mut self, cmd: Option<String>) -> &mut Self {
        self.cmd = cmd;
        self
    }

    pub fn connect(&self) -> Command {
        let mut command = Command::new("ssh");

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

        if self.verbose {
            util::print_command(&command);
        }

        command
    }
}
