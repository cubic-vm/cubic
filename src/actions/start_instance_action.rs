use crate::emulator::Emulator;
use crate::error::Error;
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
        instance_dao: &impl InstanceStore,
        instance_dir: &str,
        cache_dir: &str,
        qemu_args: &Option<String>,
        verbose: bool,
    ) -> Result<(), Error> {
        if instance_dao.is_running(&self.instance) {
            return Ok(());
        }

        let instance_dir = format!("{}/{}", instance_dir, &self.instance.name);
        let cache_dir = format!("{}/{}", cache_dir, &self.instance.name);
        setup_cloud_init(&self.instance, &cache_dir, false)?;

        let mut emulator = Emulator::from(self.instance.arch)?;
        emulator.set_cpus(self.instance.cpus);
        emulator.set_memory(self.instance.mem);
        emulator.set_console(&format!("{cache_dir}/console"));
        emulator.add_drive(&format!("{instance_dir}/machine.img"), "qcow2");
        emulator.add_drive(&format!("{cache_dir}/user-data.img"), "raw");
        emulator.set_network(&self.instance.hostfwd, self.instance.ssh_port);
        if let Some(ref args) = qemu_args {
            emulator.set_qemu_args(args);
        }
        emulator.set_verbose(verbose);
        emulator.set_pid_file(&format!("{cache_dir}/qemu.pid"));

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

        emulator.add_qmp("qmp", &format!("{cache_dir}/monitor.socket"));
        emulator.add_guest_agent("guest-agent", &format!("{cache_dir}/guest-agent.socket"));
        emulator.run()
    }

    pub fn is_done(&self) -> bool {
        PortChecker::new(self.instance.ssh_port).try_connect()
    }
}
