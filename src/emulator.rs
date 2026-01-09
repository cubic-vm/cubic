use crate::arch::Arch;
use crate::error::Error;
use crate::instance::PortForward;
use crate::util::SystemCommand;

pub struct Emulator {
    command: SystemCommand,
    verbose: bool,
}

impl Emulator {
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

    pub fn from(arch: Arch) -> Result<Emulator, Error> {
        let mut command = match arch {
            Arch::AMD64 => {
                let mut command = SystemCommand::new("qemu-system-x86_64");
                // Set machine type
                command.arg("-machine").arg("q35");
                command
            }
            Arch::ARM64 => {
                let mut command = SystemCommand::new("qemu-system-aarch64");
                // Set machine type
                command.arg("-machine").arg("virt");
                command.arg("-bios").arg("QEMU_EFI.fd");
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

        Ok(Emulator {
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

    pub fn add_qmp(&mut self, name: &str, path: &str) {
        self.command
            .args([
                "-chardev",
                &format!("socket,id={name},path={path},server=on,wait=off"),
            ])
            .args(["-mon", &format!("chardev={name},mode=control,pretty=off")]);
    }

    pub fn add_guest_agent(&mut self, name: &str, path: &str) {
        self.command
            .args([
                "-chardev",
                &format!("socket,id={name},path={path},server=on,wait=off"),
            ])
            .args(["-device", "virtio-serial"])
            .args([
                "-device",
                &format!("virtserialport,chardev={name},name=org.qemu.guest_agent.0"),
            ]);
    }

    pub fn set_console(&mut self, path: &str) {
        self.command
            .arg("-chardev")
            .arg(format!("socket,path={path},server=on,wait=off,id=console"))
            .arg("-serial")
            .arg("chardev:console");
    }

    pub fn set_network(&mut self, hostfwd: &[PortForward], ssh_port: u16) {
        let mut hostfwd_options = String::new();
        for fwd in hostfwd {
            hostfwd_options.push_str(",hostfwd=");
            hostfwd_options.push_str(&fwd.to_qemu());
        }

        self.command
            .arg("-device")
            .arg("virtio-net-pci,netdev=net0")
            .arg("-netdev")
            .arg(format!(
                "user,id=net0,hostfwd=tcp:127.0.0.1:{ssh_port}-:22{hostfwd_options}"
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

    pub fn add_search_path(&mut self, path: &str) {
        self.command.arg("-L").arg(path);
    }

    pub fn set_pid_file(&mut self, path: &str) {
        self.command.arg("-pidfile").arg(path);
    }

    pub fn run(&mut self) -> Result<(), Error> {
        self.command.arg("-daemonize");

        if self.verbose {
            println!("{}", self.command.get_command());
        }

        self.command.run()
    }
}
