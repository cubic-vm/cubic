use crate::emulator::Emulator;
use crate::error::Error;
use crate::machine::{Machine, MountPoint};
use crate::qemu::Monitor;
use crate::ssh_cmd::PortChecker;
use crate::util;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::str;
use std::thread;
use std::time::Duration;

pub const CONSOLE_COUNT: u8 = 10;
pub const USER: &str = "cubic";

#[derive(PartialEq)]
pub enum MachineState {
    Stopped,
    Starting,
    Running,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub machine: Machine,
}

pub struct MachineDao {
    pub machine_dir: String,
    pub cache_dir: String,
}

impl MachineDao {
    pub fn new() -> Result<Self, Error> {
        let machine_dir = util::get_machine_data_dir()?;
        let xdg_runtime_dir = util::get_xdg_runtime_dir()?;
        let cache_dir = format!("{xdg_runtime_dir}/cubic/machines");
        util::setup_directory_access(&machine_dir)?;
        util::setup_directory_access(&cache_dir)?;

        Result::Ok(MachineDao {
            machine_dir,
            cache_dir,
        })
    }

    pub fn get_machines(&self) -> Vec<String> {
        fs::read_dir(&self.machine_dir)
            .map_err(|_| ())
            .and_then(|entries| {
                entries
                    .collect::<Result<Vec<fs::DirEntry>, _>>()
                    .map_err(|_| ())
            })
            .and_then(|entries| {
                entries
                    .iter()
                    .map(|entry| entry.file_name().to_str().map(|x| x.to_string()).ok_or(()))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn exists(&self, name: &str) -> bool {
        Path::new(&format!("{}/{name}", self.machine_dir)).exists()
    }

    pub fn load(&self, name: &str) -> Result<Machine, Error> {
        let path = format!("{}/{name}", self.machine_dir);

        if !Path::new(&path).exists() {
            return Result::Err(Error::UnknownMachine(name.to_string()));
        }

        if !self.exists(name) {
            return Result::Err(Error::UnknownMachine(name.to_string()));
        }

        let config_path = format!("{path}/machine.yaml");
        let mut config_file = util::open_file(&config_path)?;
        Machine::deserialize(name, &mut config_file)
    }

    pub fn store(&self, machine: &Machine) -> Result<(), Error> {
        let path = format!("{}/{}", self.machine_dir, &machine.name);
        let machine_config = format!("{path}/machine.yaml");

        if Path::new(&machine_config).exists() {
            util::remove_file(&machine_config)?;
        }

        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&machine_config)
            .map_err(|_| Error::CannotCreateFile(machine_config.to_string()))?;

        machine.serialize(&mut file)
    }

    pub fn clone(&self, machine: &Machine, new_name: &str) -> Result<(), Error> {
        if self.exists(new_name) {
            Result::Err(Error::MachineAlreadyExists(new_name.to_string()))
        } else if self.is_running(machine) {
            Result::Err(Error::MachineNotStopped(machine.name.to_string()))
        } else {
            util::copy_dir(
                &format!("{}/{}", self.machine_dir, machine.name),
                &format!("{}/{new_name}", self.machine_dir),
            )
        }
    }

    pub fn rename(&self, machine: &mut Machine, new_name: &str) -> Result<(), Error> {
        if self.exists(new_name) {
            Result::Err(Error::MachineAlreadyExists(new_name.to_string()))
        } else if self.is_running(machine) {
            Result::Err(Error::MachineNotStopped(machine.name.to_string()))
        } else {
            fs::rename(
                format!("{}/{}", self.machine_dir, machine.name),
                format!("{}/{new_name}", self.machine_dir),
            )
            .map_err(Error::Io)?;
            machine.name = new_name.to_string();
            Result::Ok(())
        }
    }

    pub fn resize(&self, machine: &mut Machine, size: u64) -> Result<(), Error> {
        if self.is_running(machine) {
            Result::Err(Error::MachineNotStopped(machine.name.to_string()))
        } else if machine.disk_capacity >= size {
            Result::Err(Error::CannotShrinkDisk(machine.name.to_string()))
        } else {
            Command::new("qemu-img")
                .arg("resize")
                .arg(format!("{}/{}/machine.img", self.machine_dir, machine.name))
                .arg(size.to_string())
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(Error::Io)?
                .wait()
                .map(|_| ())
                .map_err(Error::Io)?;
            machine.disk_capacity = size;
            Result::Ok(())
        }
    }

    pub fn delete(&self, machine: &Machine) -> Result<(), Error> {
        if self.is_running(machine) {
            Result::Err(Error::MachineNotStopped(machine.name.to_string()))
        } else {
            fs::remove_dir_all(format!("{}/{}", self.cache_dir, machine.name)).ok();
            fs::remove_dir_all(format!("{}/{}", self.machine_dir, machine.name)).ok();
            Ok(())
        }
    }

    pub fn start(
        &self,
        machine: &Machine,
        qemu_args: &Option<String>,
        console: bool,
        verbose: bool,
    ) -> Result<Child, Error> {
        if self.is_running(machine) {
            return Result::Err(Error::MachineIsRunning(machine.name.to_string()));
        }

        let machine_dir = format!("{}/{}", &self.machine_dir, &machine.name);
        let cache_dir = format!("{}/{}", &self.cache_dir, &machine.name);
        util::setup_cloud_init(machine, &cache_dir, false)?;

        let mut emulator = Emulator::from(machine.name.clone())?;
        emulator.set_cpus(machine.cpus);
        emulator.set_memory(machine.mem);
        emulator.enable_kvm();
        emulator.enable_sandbox();
        emulator.set_console(&format!("{cache_dir}/console"));
        for i in 0..CONSOLE_COUNT {
            let name = format!("console{}", i);
            let path = format!("{cache_dir}/{name}");
            emulator.add_serial(&name, &path);
        }
        emulator.add_drive(&format!("{machine_dir}/machine.img"), "qcow2");
        emulator.add_drive(&format!("{cache_dir}/user-data.img"), "raw");
        emulator.set_network(&machine.hostfwd, machine.ssh_port);
        for (index, MountPoint { ref host, .. }) in machine.mounts.iter().enumerate() {
            emulator.add_mount(&format!("cubicdev{index}"), host);
        }
        emulator.set_display(machine.display, machine.gpu);
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
            emulator.add_search_path(&format!("{qemu_root}/usr/share/seabios"));
            emulator.add_search_path(&format!("{qemu_root}/usr/lib/ipxe/qemu"));
        }

        emulator.add_qmp("qmp", &format!("{cache_dir}/qmp.socket"));
        let child = emulator.run()?;

        let cache_dir = &self.cache_dir;

        if console {
            while !Path::new(&format!("{cache_dir}/console")).exists() {
                thread::sleep(Duration::new(1, 0));
            }
            util::Terminal::open(&format!("{cache_dir}/console"))?.run();
        }

        Ok(child)
    }

    pub fn stop(&self, machine: &Machine) -> Result<(), Error> {
        if !self.is_running(machine) {
            return Result::Ok(());
        }

        let mut monitor =
            Monitor::new(&format!("{}/{}/qmp.socket", self.cache_dir, &machine.name))?;
        monitor.shutdown()
    }

    pub fn get_state(&self, machine: &Machine) -> MachineState {
        if self.is_running(machine) {
            if PortChecker::new(machine.ssh_port).try_connect() {
                MachineState::Running
            } else {
                MachineState::Starting
            }
        } else {
            MachineState::Stopped
        }
    }

    pub fn is_running(&self, machine: &Machine) -> bool {
        self.get_pid(machine)
            .map(|pid| Path::new(&format!("/proc/{pid}")).exists())
            .unwrap_or(false)
    }

    fn get_pid(&self, machine: &Machine) -> Result<u64, ()> {
        let pid = fs::read_to_string(format!("{}/{}/qemu.pid", self.cache_dir, machine.name))
            .map_err(|_| ())?;

        pid.trim().parse::<u64>().map_err(|_| ())
    }
}
