use std::path::Path;

use crate::error::{Error, Result};
use crate::models::{Arch, PortForward};
use crate::util::SystemCommand;

pub struct QemuSystem {
    command: SystemCommand,
    verbose: bool,
}

impl QemuSystem {
    fn get_accelerator() -> Option<&'static str> {
        if cfg!(any(target_os = "linux", target_os = "android")) {
            Some("kvm")
        } else if cfg!(any(
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "netbsd"
        )) {
            Some("nvmm")
        } else if cfg!(any(target_os = "macos", target_os = "ios")) {
            Some("hvf")
        } else if cfg!(target_os = "windows") {
            Some("whpx")
        } else {
            None
        }
    }

    pub fn from(arch: Arch) -> Result<QemuSystem> {
        let default_binary = match arch {
            Arch::AMD64 => "qemu-system-x86_64",
            Arch::ARM64 => "qemu-system-aarch64",
        };
        let binary = std::env::var("CUBIC_QEMU").unwrap_or_else(|_| default_binary.to_owned());
        let mut command = match arch {
            Arch::AMD64 => {
                let mut command = SystemCommand::new(&binary);
                command.arg("-machine").arg("q35");
                command.arg("-smbios").arg("type=0,uefi=on");
                command
            }
            Arch::ARM64 => {
                let mut command = SystemCommand::new(&binary);
                command.arg("-machine").arg("virt");
                command
            }
        };

        // Set CPU type
        command.arg("-cpu").arg("max");

        // Enable accelerators
        if let Some(accel) = Self::get_accelerator() {
            command.arg("-accel").arg(accel);
        }
        command.arg("-accel").arg("tcg");

        // Only boot disk
        command.arg("-boot").arg("c");
        // Disable display
        command.arg("-display").arg("none");

        // Sandbox
        #[cfg(feature = "qemu-sandbox")]
        command.arg("-sandbox").arg("on");

        Ok(QemuSystem {
            command,
            verbose: false,
        })
    }

    pub fn set_verbose(&mut self, flag: bool) {
        self.verbose = flag;
    }

    pub fn add_env(&mut self, name: &str, value: &str) {
        self.command.env(name, value);
    }

    pub fn set_cpus(&mut self, cpus: u16) {
        self.command.arg("-smp").arg(cpus.to_string());
    }

    pub fn set_memory(&mut self, memory: u64) {
        self.command.arg("-m").arg(format!("{}B", memory));
    }

    pub fn set_monitor(&mut self, port: u16, instance_dir: &Path) {
        let dir = instance_dir.display();
        self.command
            .args([
                "-object",
                &format!("tls-creds-x509,id=qmp-tls,dir={dir},endpoint=server,verify-peer=yes"),
            ])
            .args([
                "-chardev",
                &format!(
                    "socket,id=qmp,host=127.0.0.1,port={port},server=on,wait=off,tls-creds=qmp-tls"
                ),
            ])
            .args(["-mon", "chardev=qmp,mode=control,pretty=off"]);
    }

    pub fn set_console(&mut self, port: u16, instance_dir: &Path) {
        let dir = instance_dir.display();
        self.command
            .args([
                "-object",
                &format!("tls-creds-x509,id=con-tls,dir={dir},endpoint=server,verify-peer=yes"),
            ])
            .arg("-chardev")
            .arg(format!(
                "socket,host=127.0.0.1,port={port},server=on,wait=off,id=console,tls-creds=con-tls"
            ))
            .arg("-serial")
            .arg("chardev:console");
    }

    pub fn set_network(&mut self, hostfwd: &[PortForward], ssh_port: u16, isolate: bool) {
        let mut hostfwd_options = String::new();
        for fwd in hostfwd {
            hostfwd_options.push_str(",hostfwd=");
            hostfwd_options.push_str(&fwd.to_qemu());
        }

        let restrict = if isolate { "on" } else { "off" };
        self.command
            .arg("-device")
            .arg("virtio-net-pci,netdev=net0")
            .arg("-netdev")
            .arg(format!(
                "user,id=net0,restrict={restrict},hostfwd=tcp:127.0.0.1:{ssh_port}-:22{hostfwd_options}"
            ));
    }

    pub fn add_drive(&mut self, path: &str, format: &str) {
        self.command
            .arg("-drive")
            .arg(format!("if=virtio,format={format},file={path}"));
    }

    pub fn set_qemu_args(&mut self, args: &str) {
        for arg in args.split(' ') {
            self.command.arg(arg);
        }
    }

    pub fn set_firmware(&mut self, path: &Path) {
        self.command
            .arg("-drive")
            .arg(format!("if=pflash,readonly=on,file={}", path.display()));
    }

    pub fn add_search_path(&mut self, path: &str) {
        self.command.arg("-L").arg(path);
    }

    pub fn set_pid_file(&mut self, path: &str) {
        self.command.arg("-pidfile").arg(path);
    }

    fn map_error(error: Error) -> Error {
        match error {
            Error::SystemCommandNotFound(program) => Error::QemuNotFound(program),
            other => other,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        if self.verbose {
            println!("{}", self.command.get_command());
        }

        self.command.run_daemonized().map_err(Self::map_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_error_translates_not_found() {
        assert!(matches!(
            QemuSystem::map_error(Error::SystemCommandNotFound("qemu-system-x86_64".to_string())),
            Error::QemuNotFound(program) if program == "qemu-system-x86_64"
        ));
    }

    #[test]
    fn test_map_error_passes_other_errors_through() {
        assert!(matches!(
            QemuSystem::map_error(Error::SystemCommandFailed(
                "cmd".to_string(),
                "boom".to_string()
            )),
            Error::SystemCommandFailed(..)
        ));
    }
}
