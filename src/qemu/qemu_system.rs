use std::path::Path;

use crate::error::{Error, Result};
use crate::models::{Arch, PortForward};
use crate::platform::System;
use crate::qemu::QemuPathBuilder;
use crate::util::SystemCommand;
use crate::view::Console;

pub struct QemuSystem {
    command: SystemCommand,
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

    pub fn from(system: &dyn System, arch: Arch) -> Result<QemuSystem> {
        let binary = format!("qemu-system-{}", arch.as_canonical_str());
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

        // Resolve the QEMU binary by name from the extended PATH.
        command.set_env("PATH", QemuPathBuilder::new(system).build());

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
        // Do not create emulated default devices (NIC, VGA, serial, parallel,
        // floppy, CD-ROM, monitor). Every device cubic needs is declared
        // explicitly, so only virtio devices plus the explicit serial console
        // remain.
        command.arg("-nodefaults");

        // Provide guest entropy via virtio-rng. Pin the builtin backend so it
        // never falls back to /dev/urandom, which is absent on Windows.
        command.arg("-object").arg("rng-builtin,id=rng0");
        command.arg("-device").arg("virtio-rng-pci,rng=rng0");
        // Allow memory reclaim via virtio-balloon.
        command.arg("-device").arg("virtio-balloon-pci");

        // Sandbox
        #[cfg(feature = "qemu-sandbox")]
        command.arg("-sandbox").arg("on");

        Ok(QemuSystem { command })
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
            .arg("virtio-net-pci,netdev=net0,romfile=")
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

    pub fn set_module_dir(&mut self, dir: &Path) {
        self.command.set_env("QEMU_MODULE_DIR", dir);
    }

    pub fn add_datadir(&mut self, dir: &Path) {
        self.command.arg("-L").arg(dir);
    }

    pub fn set_pid_file(&mut self, path: &str) {
        self.command.arg("-pidfile").arg(path);
    }

    fn map_error(error: Error) -> Error {
        match error {
            Error::SystemCommandNotFound(_) => Error::QemuNotFound,
            other => other,
        }
    }

    pub fn run(&mut self, console: &mut Console<'_>) -> Result<()> {
        console.debug(&self.command.get_command());

        self.command.run_daemonized().map_err(Self::map_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::SystemMock;

    #[test]
    fn test_map_error_translates_not_found() {
        assert!(matches!(
            QemuSystem::map_error(Error::SystemCommandNotFound(
                "qemu-system-x86_64".to_string()
            )),
            Error::QemuNotFound
        ));
    }

    #[test]
    fn test_add_datadir_appends_dash_l() {
        let mut qemu = QemuSystem::from(&SystemMock::new(), Arch::AMD64).unwrap();
        qemu.add_datadir(Path::new("/snap/cubic/current/usr/share/qemu"));
        assert!(
            qemu.command
                .get_command()
                .contains("-L /snap/cubic/current/usr/share/qemu")
        );
    }

    #[test]
    fn test_from_suppresses_emulated_default_devices() {
        let qemu = QemuSystem::from(&SystemMock::new(), Arch::AMD64).unwrap();
        let command = qemu.command.get_command();
        assert!(command.contains("-nodefaults"));
        assert!(!command.contains("-vga none"));
    }

    #[test]
    fn test_from_adds_virtio_rng_with_builtin_backend() {
        let qemu = QemuSystem::from(&SystemMock::new(), Arch::AMD64).unwrap();
        let command = qemu.command.get_command();
        assert!(command.contains("rng-builtin,id=rng0"));
        assert!(command.contains("virtio-rng-pci,rng=rng0"));
    }

    #[test]
    fn test_from_adds_virtio_balloon() {
        let qemu = QemuSystem::from(&SystemMock::new(), Arch::ARM64).unwrap();
        assert!(qemu.command.get_command().contains("virtio-balloon-pci"));
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
