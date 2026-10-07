use std::path::Path;

use crate::error::{Error, Result};
use crate::models::{Arch, PortForward};
use crate::platform::System;
use crate::qemu::QemuPathBuilder;
use crate::util::SystemCommand;

pub const NETDEV_ID: &str = "net0";
const DISK_ID: &str = "disk0";
pub const SOFTWARE_ACCEL: &str = "tcg";

pub struct QemuSystem {
    command: SystemCommand,
}

impl QemuSystem {
    pub fn get_machine(arch: Arch) -> &'static str {
        match arch {
            Arch::AMD64 => "q35,hpet=off",
            Arch::ARM64 => "virt",
        }
    }

    pub fn from(system: &dyn System, arch: Arch) -> Result<QemuSystem> {
        let binary = format!("qemu-system-{}", arch.as_canonical_str());
        let mut command = SystemCommand::new(&binary);
        command.arg("-machine").arg(Self::get_machine(arch));
        if arch == Arch::AMD64 {
            command.arg("-smbios").arg("type=0,uefi=on");
        }

        // Resolve the QEMU binary by name from the extended PATH.
        command.set_env("PATH", QemuPathBuilder::new(system).build());

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
        // Reclaim host memory via virtio-balloon. Free page reporting lets the
        // guest hand back its free pages on its own, without any host action.
        command
            .arg("-device")
            .arg("virtio-balloon-pci,free-page-reporting=on");

        Ok(QemuSystem { command })
    }

    // The CPU model follows from the accelerator. Windows takes named models
    // only, and a model richer than qemu64 faults the firmware, see #500.
    pub fn get_cpu(accel: &str) -> &'static str {
        if accel == SOFTWARE_ACCEL {
            "max"
        } else if cfg!(target_os = "windows") {
            "qemu64"
        } else {
            "host"
        }
    }

    // One accelerator only. QEMU takes the first of several that comes up,
    // which hides an accelerator that starts and then fails.
    pub fn set_accelerator(&mut self, accel: &str) {
        self.command
            .arg("-cpu")
            .arg(Self::get_cpu(accel))
            .arg("-accel")
            .arg(accel);
    }

    pub fn set_cpus(&mut self, cpus: u16) {
        self.command.arg("-smp").arg(cpus.to_string());
    }

    pub fn set_memory(&mut self, memory: u64) {
        self.command.arg("-m").arg(format!("{}B", memory));
    }

    pub fn set_monitor(&mut self, socket: &str) {
        self.command
            .arg("-chardev")
            .arg(format!("socket,id=qmp,path={socket},server=on,wait=off"))
            .args(["-mon", "chardev=qmp,mode=control,pretty=off"]);
    }

    pub fn set_console(&mut self, socket: &str) {
        self.command
            .arg("-chardev")
            .arg(format!(
                "socket,id=console,path={socket},server=on,wait=off"
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
            .arg(format!("virtio-net-pci,netdev={NETDEV_ID},romfile="))
            .arg("-netdev")
            .arg(format!(
                "user,id={NETDEV_ID},restrict={restrict},hostfwd=tcp:127.0.0.1:{ssh_port}-:22{hostfwd_options}"
            ));
    }

    // The boot index makes the firmware boot this disk only.
    pub fn add_disk(&mut self, path: &str) {
        self.command
            .arg("-drive")
            .arg(format!(
                "if=none,id={DISK_ID},format=qcow2,discard=unmap,detect-zeroes=unmap,file={path}"
            ))
            .arg("-device")
            .arg(format!("virtio-blk-pci,drive={DISK_ID},bootindex=0"));
    }

    pub fn add_iso(&mut self, path: &str) {
        self.command
            .arg("-drive")
            .arg(format!("if=virtio,format=raw,file={path}"));
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

    // A host that cannot find the binary lacks qemu, which is worth saying
    // plainly rather than reporting a command that would not start. The caller
    // runs the command, so it maps the failure through here.
    pub fn map_error(error: Error) -> Error {
        match error {
            Error::SystemCommandNotFound(_) => Error::QemuNotFound,
            other => other,
        }
    }

    pub fn build_command(self) -> SystemCommand {
        self.command
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
    fn test_add_disk_lets_guest_trim_free_space_in_the_image() {
        let mut qemu = QemuSystem::from(&SystemMock::new(), Arch::AMD64).unwrap();
        qemu.add_disk("/data/machines/test/machine.img");
        assert!(qemu.command.get_command().contains(
            "-drive if=none,id=disk0,format=qcow2,discard=unmap,detect-zeroes=unmap,file=/data/machines/test/machine.img"
        ));
    }

    #[test]
    fn test_add_disk_marks_the_disk_as_the_boot_device() {
        let mut qemu = QemuSystem::from(&SystemMock::new(), Arch::AMD64).unwrap();
        qemu.add_disk("/data/machines/test/machine.img");
        let command = qemu.command.get_command();
        assert!(command.contains("-device virtio-blk-pci,drive=disk0,bootindex=0"));
        assert!(!command.contains("-boot"));
    }

    #[test]
    fn test_add_iso_attaches_the_seed_without_discard() {
        let mut qemu = QemuSystem::from(&SystemMock::new(), Arch::AMD64).unwrap();
        qemu.add_iso("/data/machines/test/cloud-init.iso");
        let command = qemu.command.get_command();
        assert!(
            command.contains("-drive if=virtio,format=raw,file=/data/machines/test/cloud-init.iso")
        );
        assert!(!command.contains("discard"));
    }

    #[test]
    fn test_monitor_and_console_listen_on_unix_sockets() {
        let mut qemu = QemuSystem::from(&SystemMock::new(), Arch::AMD64).unwrap();
        qemu.set_monitor("/data/machines/test/monitor.sock");
        qemu.set_console("/data/machines/test/console.sock");
        let command = qemu.command.get_command();
        assert!(command.contains(
            "-chardev socket,id=qmp,path=/data/machines/test/monitor.sock,server=on,wait=off"
        ));
        assert!(command.contains(
            "-chardev socket,id=console,path=/data/machines/test/console.sock,server=on,wait=off"
        ));
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
    fn test_from_adds_virtio_balloon_that_reports_free_pages() {
        let qemu = QemuSystem::from(&SystemMock::new(), Arch::ARM64).unwrap();
        assert!(
            qemu.command
                .get_command()
                .contains("virtio-balloon-pci,free-page-reporting=on")
        );
    }

    #[test]
    fn test_build_command_names_the_binary_of_the_arch() {
        let command = QemuSystem::from(&SystemMock::new(), Arch::AMD64)
            .unwrap()
            .build_command();

        assert!(command.get_command().starts_with("qemu-system-x86_64"));
    }

    #[test]
    fn test_get_cpu_follows_the_accelerator() {
        assert_eq!(QemuSystem::get_cpu(SOFTWARE_ACCEL), "max");
        // Hardware acceleration never takes `max`, which hangs the firmware.
        assert_ne!(QemuSystem::get_cpu("kvm"), "max");
    }

    #[test]
    fn test_set_accelerator_names_one_accelerator_and_one_cpu_model() {
        let mut qemu = QemuSystem::from(&SystemMock::new(), Arch::AMD64).unwrap();
        qemu.set_accelerator(SOFTWARE_ACCEL);

        let command = qemu.command.get_command();

        assert!(command.contains("-cpu max -accel tcg"));
        assert_eq!(command.matches("-accel").count(), 1);
        assert_eq!(command.matches("-cpu").count(), 1);
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
