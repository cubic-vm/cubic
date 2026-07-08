use crate::cloudinit::UserDataImageFactory;
use crate::commands::Context;
use crate::error::{Error, Result};
use crate::fs::FS;
use crate::instance::InstanceCertGenerator;
use crate::models::{Instance, InstanceCertPaths};
use crate::qemu::{QemuFirmware, QemuInstall, QemuPathBuilder, QemuSystem};
use crate::ssh_cmd::PortChecker;
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
        console: &mut dyn Console,
    ) -> Result<()> {
        if context.get_instance_store().is_running(&self.instance) {
            return Ok(());
        }

        let env = context.get_env();
        FS::new().setup_directory_access(&env.get_instance_runtime_dir(&self.instance.name))?;
        UserDataImageFactory.create_rust(env, &self.instance)?;

        let instance_dir = PathBuf::from(env.get_instance_dir2(&self.instance.name));
        let certs = InstanceCertPaths::load(&instance_dir);
        if !certs.exists() {
            InstanceCertGenerator::new(instance_dir.clone()).generate()?;
        }

        let port_checker = PortChecker::new();
        self.instance.monitor_port = Some(port_checker.get_new_port()?);
        self.instance.console_port = Some(port_checker.get_new_port()?);
        context.get_instance_store().store(&self.instance)?;

        let mut qemu_system = QemuSystem::from(self.instance.arch)?;

        let path_builder = QemuPathBuilder::new();
        console.debug(&format!(
            "Searching for QEMU in: {}",
            path_builder
                .get_dirs()
                .iter()
                .map(|dir| dir.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ));
        let install = QemuInstall::find(path_builder.get_dirs());
        match &install {
            Some(install) => console.debug(&format!(
                "Found QEMU install at '{}'",
                install.get_prefix().display()
            )),
            None => console.debug("No QEMU install found"),
        }

        let firmware = QemuFirmware::locate(path_builder.get_dirs(), self.instance.arch)
            .ok_or(Error::QemuNotFound)?;
        console.debug(&format!("Using firmware '{}'", firmware.display()));
        qemu_system.set_firmware(&firmware);

        if let Some(install) = &install {
            match install.find_module_dir() {
                Some(module_dir) => {
                    console.debug(&format!("Using module dir '{}'", module_dir.display()));
                    qemu_system.set_module_dir(&module_dir);
                }
                None => console.debug("No module dir found"),
            }
            match install.find_datadir() {
                Some(datadir) => {
                    console.debug(&format!("Using data dir '{}'", datadir.display()));
                    qemu_system.add_datadir(&datadir);
                }
                None => console.debug("No data dir found"),
            }
        }

        qemu_system.set_cpus(self.instance.cpus);
        qemu_system.set_memory(self.instance.mem.get_bytes() as u64);
        qemu_system.set_console(self.instance.console_port.unwrap(), &instance_dir);
        qemu_system.add_drive(&env.get_instance_image_file(&self.instance.name), "qcow2");
        qemu_system.add_drive(&env.get_user_data_image_file(&self.instance.name), "raw");
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
        qemu_system.run(console)
    }

    pub fn is_done(&self) -> bool {
        PortChecker::new().is_open(self.instance.ssh_port)
    }
}
