use crate::emulator::Emulator;
use crate::error::Error;
use crate::instance::{Instance, MountPoint};
use crate::qemu::{GuestAgent, Monitor};
use crate::ssh_cmd::PortChecker;
use crate::util;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::str;

pub const CONSOLE_COUNT: u8 = 10;
pub const USER: &str = "cubic";

#[derive(PartialEq)]
pub enum InstanceState {
    Stopped,
    Starting,
    Running,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub machine: Instance,
}

pub struct InstanceDao {
    pub instance_dir: String,
    pub cache_dir: String,
}

impl InstanceDao {
    pub fn new() -> Result<Self, Error> {
        let instance_dir = util::get_instance_data_dir()?;
        let xdg_runtime_dir = util::get_xdg_runtime_dir()?;
        let cache_dir = format!("{xdg_runtime_dir}/cubic/instances");
        util::setup_directory_access(&instance_dir)?;
        util::setup_directory_access(&cache_dir)?;

        Result::Ok(InstanceDao {
            instance_dir,
            cache_dir,
        })
    }

    pub fn get_instances(&self) -> Vec<String> {
        fs::read_dir(&self.instance_dir)
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
        Path::new(&format!("{}/{name}", self.instance_dir)).exists()
    }

    pub fn load(&self, name: &str) -> Result<Instance, Error> {
        let path = format!("{}/{name}", self.instance_dir);

        if !Path::new(&path).exists() {
            return Result::Err(Error::UnknownInstance(name.to_string()));
        }

        if !self.exists(name) {
            return Result::Err(Error::UnknownInstance(name.to_string()));
        }

        let config_path = format!("{path}/machine.yaml");
        let mut config_file = util::open_file(&config_path)?;
        Instance::deserialize(name, &mut config_file)
    }

    pub fn store(&self, instance: &Instance) -> Result<(), Error> {
        let path = format!("{}/{}", self.instance_dir, &instance.name);
        let file_name = format!("{path}/machine.yaml");
        let temp_file_name = format!("{file_name}.tmp");

        let mut file = util::create_file(&temp_file_name)?;
        instance.serialize(&mut file)?;
        util::rename_file(&temp_file_name, &file_name)
    }

    pub fn clone(&self, instance: &Instance, new_name: &str) -> Result<(), Error> {
        if self.exists(new_name) {
            Result::Err(Error::InstanceAlreadyExists(new_name.to_string()))
        } else if self.is_running(instance) {
            Result::Err(Error::InstanceNotStopped(instance.name.to_string()))
        } else {
            util::copy_dir(
                &format!("{}/{}", self.instance_dir, instance.name),
                &format!("{}/{new_name}", self.instance_dir),
            )
        }
    }

    pub fn rename(&self, instance: &mut Instance, new_name: &str) -> Result<(), Error> {
        if self.exists(new_name) {
            Result::Err(Error::InstanceAlreadyExists(new_name.to_string()))
        } else if self.is_running(instance) {
            Result::Err(Error::InstanceNotStopped(instance.name.to_string()))
        } else {
            fs::rename(
                format!("{}/{}", self.instance_dir, instance.name),
                format!("{}/{new_name}", self.instance_dir),
            )
            .map_err(Error::Io)?;
            instance.name = new_name.to_string();
            Result::Ok(())
        }
    }

    pub fn resize(&self, instance: &mut Instance, size: u64) -> Result<(), Error> {
        if self.is_running(instance) {
            Result::Err(Error::InstanceNotStopped(instance.name.to_string()))
        } else if instance.disk_capacity >= size {
            Result::Err(Error::CannotShrinkDisk(instance.name.to_string()))
        } else {
            Command::new("qemu-img")
                .arg("resize")
                .arg(format!(
                    "{}/{}/machine.img",
                    self.instance_dir, instance.name
                ))
                .arg(size.to_string())
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(Error::Io)?
                .wait()
                .map(|_| ())
                .map_err(Error::Io)?;
            instance.disk_capacity = size;
            Result::Ok(())
        }
    }

    pub fn delete(&self, instance: &Instance) -> Result<(), Error> {
        if self.is_running(instance) {
            Result::Err(Error::InstanceNotStopped(instance.name.to_string()))
        } else {
            fs::remove_dir_all(format!("{}/{}", self.cache_dir, instance.name)).ok();
            fs::remove_dir_all(format!("{}/{}", self.instance_dir, instance.name)).ok();
            Ok(())
        }
    }

    pub fn start(
        &self,
        instance: &Instance,
        qemu_args: &Option<String>,
        verbose: bool,
    ) -> Result<Child, Error> {
        if self.is_running(instance) {
            return Result::Err(Error::InstanceIsRunning(instance.name.to_string()));
        }

        let instance_dir = format!("{}/{}", &self.instance_dir, &instance.name);
        let cache_dir = format!("{}/{}", &self.cache_dir, &instance.name);
        util::setup_cloud_init(instance, &cache_dir, false)?;

        let mut emulator = Emulator::from(instance.name.clone())?;
        emulator.set_cpus(instance.cpus);
        emulator.set_memory(instance.mem);
        emulator.enable_kvm();
        emulator.enable_sandbox();
        emulator.set_console(&format!("{cache_dir}/console"));
        for i in 0..CONSOLE_COUNT {
            let name = format!("console{}", i);
            let path = format!("{cache_dir}/{name}");
            emulator.add_serial(&name, &path);
        }
        emulator.add_drive(&format!("{instance_dir}/machine.img"), "qcow2");
        emulator.add_drive(&format!("{cache_dir}/user-data.img"), "raw");
        emulator.set_network(&instance.hostfwd, instance.ssh_port);
        for (index, MountPoint { ref host, .. }) in instance.mounts.iter().enumerate() {
            emulator.add_mount(&format!("cubicdev{index}"), host);
        }
        emulator.set_display(instance.display, instance.gpu);
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
        emulator.add_guest_agent("guest-agent", &format!("{cache_dir}/guest-agent.socket"));
        let child = emulator.run()?;

        Ok(child)
    }

    pub fn stop(&self, instance: &Instance) -> Result<(), Error> {
        if !self.is_running(instance) {
            return Result::Ok(());
        }

        let mut monitor =
            Monitor::new(&format!("{}/{}/qmp.socket", self.cache_dir, &instance.name))?;
        monitor.shutdown()
    }

    pub fn get_state(&self, instance: &Instance) -> InstanceState {
        if self.is_running(instance) {
            let ga = self.get_guest_agent(instance);
            if ga.and_then(|mut ga| ga.ping()).is_ok()
                || PortChecker::new(instance.ssh_port).try_connect()
            {
                InstanceState::Running
            } else {
                InstanceState::Starting
            }
        } else {
            InstanceState::Stopped
        }
    }

    pub fn is_running(&self, instance: &Instance) -> bool {
        self.get_pid(instance)
            .map(|pid| Path::new(&format!("/proc/{pid}")).exists())
            .unwrap_or(false)
    }

    pub fn get_pid(&self, instance: &Instance) -> Result<u64, ()> {
        let pid = fs::read_to_string(format!("{}/{}/qemu.pid", self.cache_dir, instance.name))
            .map_err(|_| ())?;

        pid.trim().parse::<u64>().map_err(|_| ())
    }

    pub fn get_guest_agent(&self, instance: &Instance) -> Result<GuestAgent, Error> {
        GuestAgent::new(&format!(
            "{}/{}/guest-agent.socket",
            self.cache_dir, &instance.name
        ))
    }
}
