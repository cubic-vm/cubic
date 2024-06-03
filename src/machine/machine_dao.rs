use crate::error::Error;
use crate::machine::Machine;
use crate::util;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::str;

pub const USER: &str = "cubic";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct MachineConfig {
    pub cpus: u16,
    pub mem: u64,
    pub disk_capacity: u64,
    pub ssh_port: u16,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub machine: MachineConfig,
}

pub struct MachineDao {
    pub machine_dir: String,
    pub cache_dir: String,
}

impl MachineDao {
    pub fn new() -> Result<Self, Error> {
        let home_dir = util::get_home_dir()?;
        let machine_dir = format!("{home_dir}/.local/share/cubic/machines");
        let cache_dir = format!("{home_dir}/.cache/cubic/machines");
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

        if !self.exists(name) {
            return Result::Err(Error::UnknownMachine(name.to_string()));
        }

        let config_path = format!("{path}/machine.yaml");
        let config_file = util::open_file(&config_path)?;
        let config: Config = serde_yaml::from_reader(config_file)
            .map_err(|_| Error::CannotParseFile(config_path.to_string()))?;

        Path::new(&path)
            .exists()
            .then_some(Machine {
                name: name.to_string(),
                cpus: config.machine.cpus,
                mem: config.machine.mem,
                disk_capacity: config.machine.disk_capacity,
                ssh_port: config.machine.ssh_port,
            })
            .ok_or(Error::UnknownMachine(name.to_string()))
    }

    pub fn store(&self, machine: &Machine) -> Result<(), Error> {
        let path = format!("{}/{}", self.machine_dir, &machine.name);
        let config = Config {
            machine: MachineConfig {
                cpus: machine.cpus,
                mem: machine.mem,
                disk_capacity: machine.disk_capacity,
                ssh_port: machine.ssh_port,
            },
        };

        let machine_config = format!("{path}/machine.yaml");
        if Path::new(&machine_config).exists() {
            util::remove_file(&machine_config)?;
        }

        let file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&machine_config)
            .map_err(|_| Error::CannotCreateFile(machine_config.to_string()))?;
        serde_yaml::to_writer(file, &config)
            .map_err(|_| Error::CannotWriteFile(machine_config.to_string()))
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
                .stdout(Stdio::piped())
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
            fs::remove_dir_all(format!("{}/{}", self.machine_dir, machine.name)).map_err(Error::Io)
        }
    }

    pub fn start(&self, machine: &Machine) -> Result<(), Error> {
        if self.is_running(machine) {
            return Result::Ok(());
        }

        let machine_dir = format!("{}/{}", &self.machine_dir, &machine.name);
        let cache_dir = format!("{}/{}", &self.cache_dir, &machine.name);
        util::setup_cloud_init(&machine.name, &cache_dir)?;

        let ssh_port = &machine.ssh_port;

        Command::new("bwrap")
            .arg("--ro-bind")
            .arg("/usr")
            .arg("/usr")
            .arg("--ro-bind")
            .arg("/lib64")
            .arg("/lib64")
            .arg("--ro-bind")
            .arg("/lib")
            .arg("/lib")
            .arg("--dir")
            .arg("/etc")
            .arg("--ro-bind")
            .arg("/etc/resolv.conf")
            .arg("/etc/resolv.conf")
            .arg("--dev")
            .arg("/dev")
            .arg("--dev-bind")
            .arg("/dev/kvm")
            .arg("/dev/kvm")
            .arg("--tmpfs")
            .arg("/home/cubic")
            .arg("--chdir")
            .arg("/home/cubic")
            .arg("--bind")
            .arg(&machine_dir)
            .arg(&machine_dir)
            .arg("--bind")
            .arg(&cache_dir)
            .arg(&cache_dir)
            .arg("--unshare-user")
            .arg("--unshare-ipc")
            .arg("--unshare-cgroup")
            .arg("--unshare-uts")
            .arg("--clearenv")
            .arg("--disable-userns")
            .arg("--die-with-parent")
            .arg("--new-session")
            .arg("qemu-system-amd64")
            .arg("-sandbox")
            .arg("on")
            .arg("-accel")
            .arg("kvm")
            .arg("-smp")
            .arg(machine.cpus.to_string())
            .arg("-m")
            .arg(format!("{}B", machine.mem))
            .arg("-device")
            .arg("virtio-net-pci,netdev=net0")
            .arg("-netdev")
            .arg(format!("user,id=net0,hostfwd=tcp::{ssh_port}-:22"))
            .arg("-drive")
            .arg(format!(
                "if=virtio,format=qcow2,file={machine_dir}/machine.img"
            ))
            .arg("-drive")
            .arg(format!(
                "if=virtio,file={cache_dir}/user-data.img,format=raw"
            ))
            .arg("-display")
            .arg("none")
            .arg("-serial")
            .arg(format!("unix:{cache_dir}/console.sock,server,nowait"))
            .arg("-daemonize")
            .arg("-pidfile")
            .arg(format!("{cache_dir}/qemu.pid"))
            .spawn()
            .map(|_| ())
            .map_err(|_| Error::Start(machine.name.to_string()))
    }

    pub fn stop(&self, machine: &Machine) -> Result<(), Error> {
        if !self.is_running(machine) {
            return Result::Ok(());
        }

        let pid = self
            .get_pid(machine)
            .map_err(|_| Error::Stop(machine.name.to_string()))?;

        Command::new("kill")
            .arg(pid.to_string())
            .spawn()
            .map_err(|_| Error::Stop(machine.name.to_string()))?
            .wait()
            .map(|_| ())
            .map_err(|_| Error::Stop(machine.name.to_string()))
    }

    pub fn is_running(&self, machine: &Machine) -> bool {
        Path::new(&format!("{}/{}/qemu.pid", &self.cache_dir, &machine.name)).exists()
    }

    fn get_pid(&self, machine: &Machine) -> Result<u64, ()> {
        let pid = fs::read_to_string(format!("{}/{}/qemu.pid", self.cache_dir, machine.name))
            .map_err(|_| ())?;

        pid.trim().parse::<u64>().map_err(|_| ())
    }
}
