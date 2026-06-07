use crate::cloudinit::UserDataImageFactory;
use crate::commands::Context;
use crate::emulator::Emulator;
use crate::error::Result;
use crate::firmware::FirmwareFinder;
use crate::fs::FS;
use crate::models::Instance;
use crate::ssh_cmd::PortChecker;

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

        let port_checker = PortChecker::new();
        self.instance.monitor_port = Some(port_checker.get_new_port()?);
        self.instance.console_port = Some(port_checker.get_new_port()?);
        context.get_instance_store().store(&self.instance)?;

        let snap = std::env::var("SNAP").ok();
        let snap_str = snap.as_deref();

        let mut emulator = Emulator::from(self.instance.arch)?;
        let firmware = FirmwareFinder::new(self.instance.arch, snap_str).find()?;
        emulator.set_firmware(&firmware);
        emulator.set_cpus(self.instance.cpus);
        emulator.set_memory(self.instance.mem.get_bytes() as u64);
        emulator.set_console(self.instance.console_port.unwrap());
        emulator.add_drive(&env.get_instance_image_file(&self.instance.name), "qcow2");
        emulator.add_drive(&env.get_user_data_image_file(&self.instance.name), "raw");
        emulator.set_network(
            &self.instance.hostfwd,
            self.instance.ssh_port,
            self.instance.isolate,
        );
        if let Some(args) = qemu_args {
            emulator.set_qemu_args(args);
        }
        emulator.set_verbose(verbose);
        emulator.set_pid_file(&env.get_qemu_pid_file(&self.instance.name));

        if let Some(qemu_root) = snap_str {
            emulator.add_env(
                "QEMU_MODULE_DIR",
                "/snap/cubic/current/usr/lib/x86_64-linux-gnu/qemu",
            );
            emulator.add_search_path(&format!("{qemu_root}/usr/share/qemu"));
            emulator.add_search_path(&format!("{qemu_root}/usr/share/qemu-efi-aarch64"));
            emulator.add_search_path(&format!("{qemu_root}/usr/share/seabios"));
            emulator.add_search_path(&format!("{qemu_root}/usr/lib/ipxe/qemu"));
        }

        emulator.set_monitor(self.instance.monitor_port.unwrap());
        emulator.run()
    }

    pub fn is_done(&self) -> bool {
        PortChecker::new().is_open(self.instance.ssh_port)
    }
}
