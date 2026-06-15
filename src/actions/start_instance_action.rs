use crate::cloudinit::UserDataImageFactory;
use crate::commands::Context;
use crate::error::{Error, Result};
use crate::fs::FS;
use crate::instance::InstanceCertGenerator;
use crate::models::{Instance, InstanceCertPaths};
use crate::qemu::{QemuFirmware, QemuPathBuilder, QemuSystem};
use crate::ssh_cmd::PortChecker;
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
        verbose: bool,
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
        let firmware = QemuFirmware::locate(QemuPathBuilder::new().get_dirs(), self.instance.arch)
            .ok_or(Error::QemuNotFound)?;
        qemu_system.set_firmware(&firmware);
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
        qemu_system.set_verbose(verbose);
        qemu_system.set_pid_file(&env.get_qemu_pid_file(&self.instance.name));

        qemu_system.set_monitor(self.instance.monitor_port.unwrap(), &instance_dir);
        qemu_system.run()
    }

    pub fn is_done(&self) -> bool {
        PortChecker::new().is_open(self.instance.ssh_port)
    }
}
