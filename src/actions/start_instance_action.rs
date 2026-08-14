use crate::cloudinit::CloudInitImageFactory;
use crate::commands::{Accel, Context};
use crate::error::{Error, Result};
use crate::instance::InstanceCertGenerator;
use crate::models::{Arch, Instance};
use crate::platform::System;
use crate::qemu::{
    QemuAcceleratorProbe, QemuFirmware, QemuInstall, QemuPathBuilder, QemuSystem, SOFTWARE_ACCEL,
};
use crate::ssh::PortChecker;
use crate::view::Console;
use std::path::PathBuf;

pub struct StartInstanceAction {
    instance: Instance,
}

impl StartInstanceAction {
    pub fn new(instance: &Instance) -> Self {
        Self {
            instance: instance.clone(),
        }
    }

    pub fn run(
        &mut self,
        context: &Context,
        qemu_args: &Option<String>,
        accel: Accel,
        console: &mut Console<'_>,
    ) -> Result<()> {
        if context.get_instance_store().is_running(&self.instance) {
            return Ok(());
        }

        let host_arch = Arch::get_host();
        self.check_accel_arch(accel, host_arch)?;

        let env = context.get_env();
        let system = context.get_system();
        CloudInitImageFactory.create(system, env, &self.instance)?;

        let instance_dir = PathBuf::from(env.get_instance_dir2(&self.instance.name));
        let cert_generator = InstanceCertGenerator::new(system, instance_dir.clone());
        if !cert_generator.exists() {
            cert_generator.generate()?;
        }

        self.instance.monitor_port = Some(system.bind_port()?);
        self.instance.console_port = Some(system.bind_port()?);
        context.get_instance_store().store(&self.instance)?;

        let mut qemu_system = QemuSystem::from(system, self.instance.arch)?;

        let path_builder = QemuPathBuilder::new(system);
        console.debug(&format!(
            "Searching for QEMU in: {}",
            path_builder
                .get_dirs()
                .iter()
                .map(|dir| dir.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ));
        let install = QemuInstall::find(system, path_builder.get_dirs());
        match &install {
            Some(install) => console.debug(&format!(
                "Found QEMU install at '{}'",
                install.get_prefix().display()
            )),
            None => console.debug("No QEMU install found"),
        }

        let firmware = QemuFirmware::locate(system, path_builder.get_dirs(), self.instance.arch)
            .ok_or(Error::QemuNotFound)?;
        console.debug(&format!("Using firmware '{}'", firmware.display()));
        qemu_system.set_firmware(&firmware);

        let module_dir = install
            .as_ref()
            .and_then(|install| install.find_module_dir());
        let datadir = install.as_ref().and_then(|install| install.find_datadir());
        match &module_dir {
            Some(module_dir) => {
                console.debug(&format!("Using module dir '{}'", module_dir.display()));
                qemu_system.set_module_dir(module_dir);
            }
            None => console.debug("No module dir found"),
        }
        match &datadir {
            Some(datadir) => {
                console.debug(&format!("Using data dir '{}'", datadir.display()));
                qemu_system.add_datadir(datadir);
            }
            None => console.debug("No data dir found"),
        }

        let probe = QemuAcceleratorProbe::new(
            system,
            self.instance.arch,
            &firmware,
            module_dir.as_deref(),
            datadir.as_deref(),
        );
        let accelerator = self.select_accelerator(
            accel,
            QemuAcceleratorProbe::get_host_accelerator(),
            host_arch,
            &probe,
            console,
        );
        console.debug(&format!(
            "Using accelerator '{}' with CPU model '{}'",
            accelerator,
            QemuSystem::get_cpu(accelerator)
        ));
        // The hints only help when the arch matches and the host still says no.
        if accel == Accel::Auto && accelerator == SOFTWARE_ACCEL && self.instance.arch == host_arch
        {
            // One call per line, so every line carries the warn label.
            console.warn("No hardware acceleration detected on this host.");
            for hint in QemuAcceleratorProbe::get_enable_hints(system) {
                console.warn(hint);
            }
        }
        qemu_system.set_accelerator(accelerator);

        qemu_system.set_cpus(self.instance.cpus);
        qemu_system.set_memory(self.instance.mem.get_bytes() as u64);
        qemu_system.set_console(self.instance.console_port.unwrap(), &instance_dir);
        qemu_system.add_drive(&env.get_instance_image_file(&self.instance.name), "qcow2");
        qemu_system.add_drive(&env.get_cloud_init_file(&self.instance.name), "raw");
        qemu_system.set_network(
            &self.instance.hostfwd,
            self.instance.ssh_port,
            self.instance.isolate,
        );
        if let Some(args) = qemu_args {
            qemu_system.set_qemu_args(args);
        }
        qemu_system.set_pid_file(&env.get_qemu_pid_file(&self.instance.name));

        qemu_system.set_monitor(self.instance.monitor_port.unwrap(), &instance_dir);

        let command = qemu_system.build_command();
        console.debug(&command.get_command());
        system
            .spawn_command(&command)
            .map_err(QemuSystem::map_error)
    }

    // An accelerator runs guest code on the host CPU, so it needs both archs to
    // be the same. Say so before the start does any work.
    fn check_accel_arch(&self, accel: Accel, host_arch: Arch) -> Result<()> {
        if accel == Accel::On && self.instance.arch != host_arch {
            return Err(Error::ArchMismatch(
                self.instance.name.clone(),
                self.instance.arch,
                host_arch,
            ));
        }
        Ok(())
    }

    // `on` and `off` overrule the probe and go straight to QEMU. A guest arch
    // that differs from the host arch rules out every hardware accelerator, so
    // it needs no probe.
    fn select_accelerator(
        &self,
        accel: Accel,
        host_accel: &'static str,
        host_arch: Arch,
        probe: &QemuAcceleratorProbe,
        console: &mut Console<'_>,
    ) -> &'static str {
        match accel {
            Accel::Off => SOFTWARE_ACCEL,
            Accel::On => host_accel,
            Accel::Auto if self.instance.arch != host_arch => {
                console.debug(&format!(
                    "No hardware acceleration: guest arch {} differs from host arch {host_arch}",
                    self.instance.arch
                ));
                SOFTWARE_ACCEL
            }
            Accel::Auto => match probe.detect(host_accel) {
                Ok(()) => host_accel,
                Err(reason) => {
                    console.debug(&format!("No hardware acceleration: {reason}"));
                    SOFTWARE_ACCEL
                }
            },
        }
    }

    pub fn is_done(&self, system: &dyn System) -> bool {
        PortChecker::new().is_open(system, self.instance.ssh_port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::SystemMock;
    use std::path::Path;

    fn build_action(guest_arch: Arch) -> StartInstanceAction {
        StartInstanceAction::new(&Instance {
            name: "test".to_string(),
            arch: guest_arch,
            ..Instance::default()
        })
    }

    fn select_for(
        system: &SystemMock,
        accel: Accel,
        guest_arch: Arch,
        host_arch: Arch,
    ) -> &'static str {
        let mut console = Console::new(system);
        let probe = QemuAcceleratorProbe::new(
            system,
            guest_arch,
            Path::new("/usr/share/qemu/OVMF.fd"),
            None,
            None,
        );
        build_action(guest_arch).select_accelerator(accel, "kvm", host_arch, &probe, &mut console)
    }

    fn select(system: &SystemMock, accel: Accel) -> &'static str {
        select_for(system, accel, Arch::AMD64, Arch::AMD64)
    }

    #[test]
    fn test_accel_off_runs_on_tcg_without_asking_qemu() {
        let system = SystemMock::new();

        assert_eq!(select(&system, Accel::Off), "tcg");
        assert!(system.get_executed_commands().is_empty());
    }

    #[test]
    fn test_accel_on_takes_the_accelerator_without_asking_qemu() {
        let system = SystemMock::new();

        assert_eq!(select(&system, Accel::On), "kvm");
        assert!(system.get_executed_commands().is_empty());
    }

    #[test]
    fn test_accel_auto_asks_qemu() {
        let system = SystemMock::new();

        let accelerator = select(&system, Accel::Auto);

        assert_eq!(accelerator, "tcg");
        assert_eq!(system.get_executed_commands().len(), 1);
    }

    #[test]
    fn test_accel_on_fails_when_the_guest_arch_differs_from_the_host() {
        let result = build_action(Arch::ARM64).check_accel_arch(Accel::On, Arch::AMD64);

        assert!(matches!(
            result,
            Err(Error::ArchMismatch(ref name, Arch::ARM64, Arch::AMD64)) if name == "test"
        ));
    }

    #[test]
    fn test_accel_on_passes_the_check_on_the_host_arch() {
        assert!(
            build_action(Arch::AMD64)
                .check_accel_arch(Accel::On, Arch::AMD64)
                .is_ok()
        );
    }

    #[test]
    fn test_accel_auto_passes_the_check_on_a_foreign_arch() {
        assert!(
            build_action(Arch::ARM64)
                .check_accel_arch(Accel::Auto, Arch::AMD64)
                .is_ok()
        );
    }

    #[test]
    fn test_accel_auto_skips_the_probe_on_a_foreign_arch() {
        let system = SystemMock::new();

        let accelerator = select_for(&system, Accel::Auto, Arch::ARM64, Arch::AMD64);

        assert_eq!(accelerator, "tcg");
        assert!(system.get_executed_commands().is_empty());
    }
}
