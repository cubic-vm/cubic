use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::error::{Error, Result};
use crate::models::Arch;
use crate::platform::System;
use crate::qemu::{QemuSystem, SOFTWARE_ACCEL};
use crate::util::SystemCommand;

// Only reached by a QEMU that hangs, since one that cannot use an accelerator
// says so within milliseconds and one that can is up just as fast.
const PROBE_TIMEOUT: Duration = Duration::from_secs(3);

// QEMU greets a QMP client once the machine stands, which is after the
// accelerator is initialised and the firmware is loaded.
const QMP_GREETING: &str = "\"QMP\"";

// Asks QEMU whether an accelerator works on this host, by starting it as the
// instance would and killing it once it is up.
pub struct QemuAcceleratorProbe<'a> {
    system: &'a dyn System,
    arch: Arch,
    firmware: PathBuf,
    module_dir: Option<PathBuf>,
    datadir: Option<PathBuf>,
}

impl<'a> QemuAcceleratorProbe<'a> {
    // Takes the firmware, module dir and data dir of the instance, or the probe
    // answers for a machine the real start never builds.
    pub fn new(
        system: &'a dyn System,
        arch: Arch,
        firmware: &Path,
        module_dir: Option<&Path>,
        datadir: Option<&Path>,
    ) -> Self {
        Self {
            system,
            arch,
            firmware: firmware.to_path_buf(),
            module_dir: module_dir.map(Path::to_path_buf),
            datadir: datadir.map(Path::to_path_buf),
        }
    }

    pub fn get_host_accelerator() -> &'static str {
        if cfg!(any(target_os = "linux", target_os = "android")) {
            "kvm"
        } else if cfg!(any(target_os = "macos", target_os = "ios")) {
            "hvf"
        } else if cfg!(target_os = "windows") {
            "whpx"
        } else if cfg!(any(
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "netbsd"
        )) {
            "nvmm"
        } else {
            SOFTWARE_ACCEL
        }
    }

    // What the user can turn on to get hardware acceleration on this host. A
    // snap reaches /dev/kvm through an interface that is not connected on
    // install, which hides the device whatever the user belongs to.
    pub fn get_enable_hints(system: &dyn System) -> Vec<&'static str> {
        let mut hints = Vec::new();
        if system.read_env_var("SNAP").is_some() {
            hints.push("Connect the kvm interface with sudo snap connect cubic:kvm.");
        }
        hints.push(Self::get_platform_hint());
        hints
    }

    fn get_platform_hint() -> &'static str {
        if cfg!(any(target_os = "macos", target_os = "ios")) {
            "Install QEMU from Homebrew to get the hypervisor entitlement."
        } else if cfg!(target_os = "windows") {
            "Enable virtualization in UEFI/BIOS and the Windows Hypervisor Platform."
        } else if cfg!(any(
            target_os = "freebsd",
            target_os = "dragonfly",
            target_os = "openbsd",
            target_os = "netbsd"
        )) {
            "Enable virtualization in UEFI/BIOS and load the nvmm module."
        } else {
            "Enable virtualization in UEFI/BIOS and add your user to the kvm group."
        }
    }

    // Reports what QEMU said, not the generic wrapper around a failed command.
    pub fn detect(&self, accel: &str) -> std::result::Result<(), String> {
        let command = self
            .build_command(accel)
            .map_err(|error| error.to_string())?;
        self.system
            .run_command_until_output(&command, QMP_GREETING, PROBE_TIMEOUT)
            .map_err(|error| match error {
                Error::SystemCommandFailed(_, stderr) if stderr.trim().is_empty() => {
                    "QEMU did not come up".to_string()
                }
                Error::SystemCommandFailed(_, stderr) => stderr.trim().to_string(),
                other => other.to_string(),
            })
    }

    // The machine of the probe is the machine of the start, firmware included,
    // or the answer belongs to a different machine than the one that follows.
    fn build_command(&self, accel: &str) -> Result<SystemCommand> {
        let mut qemu = QemuSystem::from(self.system, self.arch)?;
        qemu.set_firmware(&self.firmware);
        if let Some(module_dir) = &self.module_dir {
            qemu.set_module_dir(module_dir);
        }
        if let Some(datadir) = &self.datadir {
            qemu.add_datadir(datadir);
        }
        qemu.set_accelerator(accel);

        let mut command = qemu.build_command();
        command.arg("-qmp").arg("stdio");
        Ok(command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::SystemMock;

    const FIRMWARE: &str = "/usr/share/qemu/OVMF.fd";

    fn build_probe(system: &SystemMock, arch: Arch) -> QemuAcceleratorProbe<'_> {
        QemuAcceleratorProbe::new(system, arch, Path::new(FIRMWARE), None, None)
    }

    // The probe carries the whole machine, cpu model and firmware included, so
    // the expected command comes from the builder rather than from a copy of it
    // that can drift.
    fn build_probe_command(arch: Arch, accel: &str) -> String {
        let system = SystemMock::new();
        build_probe(&system, arch)
            .build_command(accel)
            .unwrap()
            .get_command()
    }

    #[test]
    fn test_get_enable_hints_asks_a_snap_to_connect_the_interface_first() {
        let system = SystemMock::new().add_env_var("SNAP", "/snap/cubic/current");

        let hints = QemuAcceleratorProbe::get_enable_hints(&system);

        assert_eq!(hints.len(), 2);
        assert!(hints[0].contains("snap connect cubic:kvm"));
    }

    #[test]
    fn test_get_enable_hints_names_the_platform_alone_off_snap() {
        let hints = QemuAcceleratorProbe::get_enable_hints(&SystemMock::new());

        assert_eq!(hints.len(), 1);
        assert!(!hints[0].contains("snap"));
    }

    #[test]
    fn test_detect_accepts_an_accelerator_that_comes_up() {
        let system =
            SystemMock::new().add_command_output(&build_probe_command(Arch::AMD64, "kvm"), b"");

        assert!(build_probe(&system, Arch::AMD64).detect("kvm").is_ok());
    }

    #[test]
    fn test_detect_reports_what_qemu_said() {
        let system = SystemMock::new()
            .add_failing_command(&build_probe_command(Arch::AMD64, "kvm"), "no kvm");

        assert_eq!(
            build_probe(&system, Arch::AMD64).detect("kvm"),
            Err("no kvm".to_string())
        );
    }

    #[test]
    fn test_detect_names_a_qemu_that_says_nothing() {
        let system =
            SystemMock::new().add_failing_command(&build_probe_command(Arch::AMD64, "kvm"), "  ");

        assert_eq!(
            build_probe(&system, Arch::AMD64).detect("kvm"),
            Err("QEMU did not come up".to_string())
        );
    }

    #[test]
    fn test_detect_probes_the_binary_of_the_arch() {
        let system = SystemMock::new();

        build_probe(&system, Arch::ARM64).detect("kvm").ok();

        assert!(system.get_executed_commands()[0].starts_with("qemu-system-aarch64 -machine virt"));
    }

    #[test]
    fn test_detect_probes_with_the_firmware_of_the_start() {
        let system = SystemMock::new();

        build_probe(&system, Arch::AMD64).detect("kvm").ok();

        assert!(
            system.get_executed_commands()[0]
                .contains(&format!("-drive if=pflash,readonly=on,file={FIRMWARE}"))
        );
    }

    #[test]
    fn test_detect_asks_qemu_to_report_that_the_machine_is_up() {
        let system = SystemMock::new();

        build_probe(&system, Arch::AMD64).detect("kvm").ok();

        assert!(system.get_executed_commands()[0].ends_with("-qmp stdio"));
    }

    #[test]
    fn test_detect_probes_with_the_data_dir_of_the_install() {
        let system = SystemMock::new();

        QemuAcceleratorProbe::new(
            &system,
            Arch::AMD64,
            Path::new(FIRMWARE),
            Some(Path::new("/usr/lib/qemu")),
            Some(Path::new("/usr/share/qemu")),
        )
        .detect("kvm")
        .ok();

        assert!(system.get_executed_commands()[0].contains("-L /usr/share/qemu"));
    }
}
