use crate::error::Error;
use crate::util;

use std::process::{Command, Stdio};

pub struct Emulator {
    name: String,
    command: Command,
    verbose: bool,
}

impl Emulator {
    pub fn from(name: String) -> Result<Emulator, Error> {
        let command = Command::new("qemu-system-x86_64");
        Ok(Emulator {
            name,
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

    pub fn enable_kvm(&mut self) {
        if util::has_kvm() {
            self.command.arg("-accel").arg("kvm");
        } else {
            println!("WARNING: No KVM support detected");
        }
    }

    pub fn enable_sandbox(&mut self) {
        self.command.arg("-sandbox").arg("on");
    }

    pub fn add_serial(&mut self, name: &str, path: &str) {
        self.command
            .arg("-device")
            .arg("virtio-serial")
            .arg("-chardev")
            .arg(format!("socket,path={path},server=on,wait=off,id={name}"))
            .arg("-device")
            .arg(format!("virtconsole,chardev={name}"));
    }

    pub fn add_qmp(&mut self, name: &str, path: &str) {
        self.command
            .args([
                "-chardev",
                &format!("socket,id={name},path={path},server=on,wait=off"),
            ])
            .args(["-mon", &format!("chardev={name},mode=control,pretty=off")]);
    }

    pub fn set_console(&mut self, path: &str) {
        self.command
            .arg("-chardev")
            .arg(format!("socket,path={path},server=on,wait=off,id=console"))
            .arg("-serial")
            .arg("chardev:console");
    }

    pub fn set_network(&mut self, hostfwd: &[String], ssh_port: u16) {
        let mut hostfwd_options = String::new();
        for fwd in hostfwd {
            hostfwd_options.push_str(",hostfwd=");
            hostfwd_options.push_str(fwd);
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

    pub fn add_mount(&mut self, name: &str, path: &str) {
        self.command
            .arg("-fsdev")
            .arg(format!(
                "local,security_model=mapped,id={name}_dev,multidevs=remap,path={path}"
            ))
            .arg("-device")
            .arg(format!(
                "virtio-9p-pci,id={name},fsdev={name}_dev,mount_tag=cubic{name}"
            ));
    }

    pub fn set_display(&mut self, display: bool, gpu: bool) {
        self.command.arg("-display");
        if display {
            self.command.arg("gtk,gl=on");
            self.command.arg("-device");
            if gpu {
                self.command.arg("virtio-gpu-gl");
            } else {
                self.command.arg("virtio-gpu");
            }
        } else {
            self.command.arg("none");
        }
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
            util::print_command(&self.command);
        } else {
            self.command
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());
        }

        self.command
            .spawn()
            .map(|_| ())
            .map_err(|_| Error::Start(self.name.clone()))?;

        Ok(())
    }
}
