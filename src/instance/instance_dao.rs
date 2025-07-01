use crate::emulator::Emulator;
use crate::error::Error;
use crate::fs::FS;
use crate::instance::{Instance, InstanceState, InstanceStore, MountPoint};
use crate::qemu::{GuestAgent, Monitor};
use crate::ssh_cmd::PortChecker;
use crate::util;
use serde::{Deserialize, Serialize};
use std::fs::DirEntry;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::str;

pub const USER: &str = "cubic";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub machine: Instance,
}

pub struct InstanceDao {
    fs: FS,
    pub instance_dir: String,
    pub cache_dir: String,
}

impl InstanceDao {
    pub fn new() -> Result<Self, Error> {
        let fs = FS::new();
        let instance_dir = util::get_instance_data_dir()?;
        let xdg_runtime_dir = util::get_xdg_runtime_dir()?;
        let cache_dir = format!("{xdg_runtime_dir}/cubic/instances");
        fs.setup_directory_access(&instance_dir)?;
        fs.setup_directory_access(&cache_dir)?;

        Result::Ok(InstanceDao {
            fs,
            instance_dir,
            cache_dir,
        })
    }
}

impl InstanceStore for InstanceDao {
    fn get_instances(&self) -> Vec<String> {
        self.fs
            .read_dir(&self.instance_dir)
            .map_err(|_| ())
            .and_then(|entries| {
                entries
                    .collect::<Result<Vec<DirEntry>, _>>()
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

    fn exists(&self, name: &str) -> bool {
        Path::new(&format!("{}/{name}", self.instance_dir)).exists()
    }

    fn load(&self, name: &str) -> Result<Instance, Error> {
        if !self.exists(name) {
            return Result::Err(Error::UnknownInstance(name.to_string()));
        }

        self.fs
            .open_file(&format!("{}/{name}/machine.yaml", self.instance_dir))
            .and_then(|mut file| Instance::deserialize(name, &mut file))
            .or(Ok(Instance {
                name: name.to_string(),
                user: USER.to_string(),
                cpus: 1,
                mem: util::human_readable_to_bytes("1G").unwrap(),
                disk_capacity: util::human_readable_to_bytes("1G").unwrap(),
                ssh_port: util::generate_random_ssh_port(),
                ..Instance::default()
            }))
    }

    fn store(&self, instance: &Instance) -> Result<(), Error> {
        let path = format!("{}/{}", self.instance_dir, &instance.name);
        let file_name = format!("{path}/machine.yaml");
        let temp_file_name = format!("{file_name}.tmp");

        let mut file = self.fs.create_file(&temp_file_name)?;
        instance.serialize(&mut file)?;
        self.fs.rename_file(&temp_file_name, &file_name)
    }

    fn clone(&self, instance: &Instance, new_name: &str) -> Result<(), Error> {
        if self.exists(new_name) {
            Result::Err(Error::InstanceAlreadyExists(new_name.to_string()))
        } else if self.is_running(instance) {
            Result::Err(Error::InstanceNotStopped(instance.name.to_string()))
        } else {
            self.fs.copy_dir(
                &format!("{}/{}", self.instance_dir, instance.name),
                &format!("{}/{new_name}", self.instance_dir),
            )
        }
    }

    fn rename(&self, instance: &mut Instance, new_name: &str) -> Result<(), Error> {
        if self.exists(new_name) {
            Result::Err(Error::InstanceAlreadyExists(new_name.to_string()))
        } else if self.is_running(instance) {
            Result::Err(Error::InstanceNotStopped(instance.name.to_string()))
        } else {
            self.fs.rename_file(
                &format!("{}/{}", self.instance_dir, instance.name),
                &format!("{}/{new_name}", self.instance_dir),
            )?;
            instance.name = new_name.to_string();
            Result::Ok(())
        }
    }

    fn resize(&self, instance: &mut Instance, size: u64) -> Result<(), Error> {
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

    fn delete(&self, instance: &Instance) -> Result<(), Error> {
        if self.is_running(instance) {
            Result::Err(Error::InstanceNotStopped(instance.name.to_string()))
        } else {
            self.fs
                .remove_dir(&format!("{}/{}", self.cache_dir, instance.name))
                .ok();
            self.fs
                .remove_dir(&format!("{}/{}", self.instance_dir, instance.name))
                .ok();
            Ok(())
        }
    }

    fn start(
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

        let mut emulator = Emulator::from(instance.name.clone(), instance.arch)?;
        emulator.set_cpus(instance.cpus);
        emulator.set_memory(instance.mem);
        emulator.set_console(&format!("{cache_dir}/console"));
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
            emulator.add_search_path(&format!("{qemu_root}/usr/share/qemu-efi-aarch64"));
            emulator.add_search_path(&format!("{qemu_root}/usr/share/seabios"));
            emulator.add_search_path(&format!("{qemu_root}/usr/lib/ipxe/qemu"));
        }

        emulator.add_qmp("qmp", &format!("{cache_dir}/monitor.socket"));
        emulator.add_guest_agent("guest-agent", &format!("{cache_dir}/guest-agent.socket"));

        emulator.add_virtio_serial("sh_serial");
        emulator.run()
    }

    fn stop(&self, instance: &Instance) -> Result<(), Error> {
        if !self.is_running(instance) {
            return Result::Ok(());
        }

        self.get_monitor(instance)?.shutdown()
    }

    fn get_state(&self, instance: &Instance) -> InstanceState {
        if self.is_running(instance) {
            if PortChecker::new(instance.ssh_port).try_connect() {
                InstanceState::Running
            } else {
                InstanceState::Starting
            }
        } else {
            InstanceState::Stopped
        }
    }

    fn is_running(&self, instance: &Instance) -> bool {
        self.get_pid(instance)
            .map(|pid| Path::new(&format!("/proc/{pid}")).exists())
            .unwrap_or(false)
    }

    fn get_pid(&self, instance: &Instance) -> Result<u64, ()> {
        let pid = self
            .fs
            .read_file_to_string(&format!("{}/{}/qemu.pid", self.cache_dir, instance.name))
            .map_err(|_| ())?;

        pid.trim().parse::<u64>().map_err(|_| ())
    }

    fn get_monitor(&self, instance: &Instance) -> Result<Monitor, Error> {
        Monitor::new(&format!("{}/{}", self.cache_dir, &instance.name))
    }

    fn get_guest_agent(&self, instance: &Instance) -> Result<GuestAgent, Error> {
        GuestAgent::new(&format!(
            "{}/{}/guest-agent.socket",
            self.cache_dir, &instance.name
        ))
    }
}
