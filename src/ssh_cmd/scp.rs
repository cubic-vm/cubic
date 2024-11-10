use crate::util;
use std::path::Path;
use std::process::Command;

#[derive(Default)]
pub struct Scp {
    root_dir: String,
    known_hosts_file: Option<String>,
    private_keys: Vec<String>,
    port: Option<u16>,
    args: String,
    verbose: bool,
}

impl Scp {
    pub fn new() -> Self {
        Scp::default()
    }

    pub fn set_root_dir(&mut self, root_dir: &str) -> &mut Self {
        self.root_dir = root_dir.to_string();
        self
    }

    pub fn set_known_hosts_file(&mut self, path: Option<String>) -> &mut Self {
        self.known_hosts_file = path;
        self
    }

    pub fn set_private_keys(&mut self, private_keys: Vec<String>) -> &mut Self {
        self.private_keys = private_keys;
        self
    }

    pub fn set_port(&mut self, port: Option<u16>) -> &mut Self {
        self.port = port;
        self
    }

    pub fn set_args(&mut self, args: &str) -> &mut Self {
        self.args = args.to_string();
        self
    }

    pub fn set_verbose(&mut self, verbose: bool) -> &mut Self {
        self.verbose = verbose;
        self
    }

    pub fn copy(&self, from: &str, to: &str) -> Command {
        let mut command = Command::new(format!("{}/usr/bin/scp", self.root_dir));

        if let Some(ref known_hosts_file) = self.known_hosts_file {
            Path::new(known_hosts_file)
                .parent()
                .and_then(|dir| dir.to_str())
                .map(util::create_dir);

            command.arg(format!("-oUserKnownHostsFile={known_hosts_file}"));
        }

        command
            .arg("-r")
            .args(self.port.map(|port| format!("-P{port}")).as_slice())
            .arg(format!("-S{}/usr/bin/ssh", self.root_dir))
            .args(
                self.private_keys
                    .iter()
                    .map(|key| format!("-i{key}"))
                    .collect::<Vec<_>>(),
            )
            .args(self.args.split(' ').filter(|item| !item.is_empty()))
            .arg(from)
            .arg(to);

        if self.verbose {
            util::print_command(&command);
        }

        command
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    #[test]
    fn test_scp_minimal() {
        let cmd = Scp::new().copy("/from/file", "/to/file");
        let args: Vec<&OsStr> = cmd.get_args().collect();

        assert_eq!(cmd.get_program(), "/usr/bin/scp");
        assert_eq!(args, &["-r", "-S/usr/bin/ssh", "/from/file", "/to/file"]);
    }

    #[test]
    fn test_scp_minimal_snap() {
        let cmd = Scp::new()
            .set_root_dir("/snap/cubic/current")
            .copy("/from/file", "/to/file");
        let args: Vec<&OsStr> = cmd.get_args().collect();

        assert_eq!(cmd.get_program(), "/snap/cubic/current/usr/bin/scp");
        assert_eq!(
            args,
            &[
                "-r",
                "-S/snap/cubic/current/usr/bin/ssh",
                "/from/file",
                "/to/file"
            ]
        );
    }

    #[test]
    fn test_scp_advanced() {
        let cmd = Scp::new()
            .set_root_dir("/snap/cubic/current")
            .set_verbose(true)
            .set_known_hosts_file(Some("/home/test/.ssh/known_hosts".to_string()))
            .set_private_keys(vec![
                "/home/cubic/.ssh/id_rsa".to_string(),
                "/home/cubic/.ssh/id_ed25519".to_string(),
            ])
            .set_port(Some(10000))
            .set_args("-myarg1 -myarg2 -myarg3")
            .copy("/from/file", "/to/file");
        let args: Vec<&OsStr> = cmd.get_args().collect();

        assert_eq!(cmd.get_program(), "/snap/cubic/current/usr/bin/scp");
        assert_eq!(
            args,
            &[
                "-oUserKnownHostsFile=/home/test/.ssh/known_hosts",
                "-r",
                "-P10000",
                "-S/snap/cubic/current/usr/bin/ssh",
                "-i/home/cubic/.ssh/id_rsa",
                "-i/home/cubic/.ssh/id_ed25519",
                "-myarg1",
                "-myarg2",
                "-myarg3",
                "/from/file",
                "/to/file"
            ]
        );
    }
}
