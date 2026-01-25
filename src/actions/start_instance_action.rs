use crate::emulator::Emulator;
use crate::env::Environment;
use crate::error::Error;
use crate::fs::FS;
use crate::instance::{Instance, InstanceStore};
use crate::ssh_cmd::PortChecker;
use crate::util::setup_cloud_init;

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
        instance_dao: &dyn InstanceStore,
        env: &Environment,
        qemu_args: &Option<String>,
        verbose: bool,
    ) -> Result<(), Error> {
        if instance_dao.is_running(&self.instance) {
            return Ok(());
        }

        FS::new().setup_directory_access(&env.get_instance_runtime_dir(&self.instance.name))?;
        setup_cloud_init(env, &self.instance)?;

        let mut emulator = Emulator::from(self.instance.arch)?;
        emulator.set_cpus(self.instance.cpus);
        emulator.set_memory(self.instance.mem.get_bytes() as u64);
        emulator.set_console(&env.get_console_file(&self.instance.name));
        emulator.add_drive(&env.get_instance_image_file(&self.instance.name), "qcow2");
        emulator.add_drive(&env.get_user_data_image_file(&self.instance.name), "raw");
        emulator.set_network(&self.instance.hostfwd, self.instance.ssh_port);
        if let Some(args) = qemu_args {
            emulator.set_qemu_args(args);
        }
        emulator.set_verbose(verbose);
        emulator.set_pid_file(&env.get_qemu_pid_file(&self.instance.name));

        if let Ok(qemu_root) = std::env::var("SNAP") {
            emulator.add_env(
                "QEMU_MODULE_DIR",
                "/snap/cubic/current/usr/lib/x86_64-linux-gnu/qemu",
            );
            emulator.add_search_path(&format!("{qemu_root}/usr/share/qemu"));
            emulator.add_search_path(&format!("{qemu_root}/usr/share/qemu-efi-aarch64"));
            emulator.add_search_path(&format!("{qemu_root}/usr/share/seabios"));
            emulator.add_search_path(&format!("{qemu_root}/usr/lib/ipxe/qemu"));
        }

        emulator.add_qmp("qmp", &env.get_monitor_file(&self.instance.name));
        emulator.add_guest_agent(
            "guest-agent",
            &env.get_guest_agent_file(&self.instance.name),
        );
        emulator.run()
    }

    pub fn is_done(&self) -> bool {
        PortChecker::new().is_open(self.instance.ssh_port)
    }
}
