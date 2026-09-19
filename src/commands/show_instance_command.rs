use crate::actions::LoadInstanceAction;
use crate::commands::{self, Command};
use crate::error::{Error, Result};
use crate::ssh::HostKeyChecker;
use crate::util;
use crate::view::MapView;
use clap::Parser;

/// Show a VM instance
#[derive(Parser)]
pub struct ShowInstanceCommand {
    #[clap(flatten)]
    pub instance: commands::InstanceArg,

    #[clap(flatten)]
    pub all: commands::AllInfoArg,
}

impl Command for ShowInstanceCommand {
    async fn run(&self, context: &commands::Context) -> Result<u8> {
        let env = context.get_env();
        let instance_store = context.get_instance_store();

        if !instance_store.exists(self.instance.value.as_str()) {
            return Err(Error::UnknownInstance(self.instance.value.to_string()));
        }

        let instance = LoadInstanceAction::new().run(context, self.instance.value.as_str())?;
        let ssh_key = env.get_ssh_private_key_file(&instance.name);

        let mut view = MapView::new();
        view.add(
            "Running",
            util::to_yes_no(instance_store.is_running(&instance)),
        );
        view.add("Arch", &instance.arch.to_string());
        view.add("vCPUs", &instance.cpus.to_string());
        view.add("Memory", &instance.mem.to_size());
        if let Some(disk_used) = &instance.disk_used {
            view.add("Disk Used", &disk_used.to_size());
        }
        view.add("Disk Total", &instance.disk_capacity.to_size());
        view.add("User", instance.user.as_str());
        view.add("Isolated", util::to_yes_no(instance.isolate));

        // Port forwarding
        for (index, rule) in instance.hostfwd.iter().enumerate() {
            let key = if index == 0 { "Forward" } else { "" };
            view.add(key, &rule.to_string());
        }

        for (index, snapshot) in instance.snapshots.iter().enumerate() {
            let key = if index == 0 { "Snapshots" } else { "" };
            view.add(key, &snapshot.name);
        }

        if self.all.value {
            if let Some(pid) = instance_store.get_pid(&instance) {
                view.add("PID", &pid.to_string());
            }
            view.add("SSH Port", &instance.ssh_port.to_string());
            if let Some(monitor_port) = instance.monitor_port {
                view.add("Monitor Port", &monitor_port.to_string());
            }
            if let Some(console_port) = instance.console_port {
                view.add("Console Port", &console_port.to_string());
            }
            view.add("Disk Image", &env.get_instance_image_file(&instance.name));
            view.add("Config", &env.get_instance_toml_config_file(&instance.name));
            view.add("SSH Key", &ssh_key);
            if let Some(host_key) = &instance.ssh_host_key {
                view.add(
                    "SSH Host Key",
                    &HostKeyChecker::new().get_fingerprint(host_key),
                );
            }
            view.add(
                "SSH",
                &format!(
                    "ssh -i {} -p {} {}@localhost",
                    ssh_key, instance.ssh_port, instance.user
                ),
            );
        }

        view.print(context.get_console());

        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::InstanceStoreMock;
    use crate::models::{Arch, DataSize, Environment, Instance, InstanceName, Snapshot, UserName};
    use crate::platform::{System, SystemMock};
    use crate::view::Console;
    use std::path::PathBuf;
    use std::str::FromStr;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_show_basic_fields() {
        let system = Arc::new(SystemMock::new());
        let env = Environment::new(
            UserName::from_str("myuser").unwrap(),
            String::new(),
            String::new(),
        );
        let instance_store = InstanceStoreMock::new(vec![Instance {
            name: "test".to_string(),
            arch: Arch::AMD64,
            user: UserName::from_str("myuser").unwrap(),
            cpus: 1,
            mem: DataSize::new(1024),
            disk_capacity: DataSize::new(1048576),
            ssh_port: 9000,
            hostfwd: vec!["127.0.0.1:4000:40/tcp".parse().unwrap()],
            ..Instance::default()
        }]);
        let context = commands::Context::new(
            Arc::new(SystemMock::new()),
            Console::new(Arc::clone(&system) as Arc<dyn System>),
            env,
            Box::new(instance_store),
        );

        ShowInstanceCommand {
            instance: InstanceName::from_str("test").unwrap().into(),
            all: false.into(),
        }
        .run(&context)
        .await
        .unwrap();

        assert_eq!(
            system.get_output(),
            "\
Running:    no
Arch:       amd64
vCPUs:      1
Memory:     1024 B
Disk Total: 1024 K
User:       myuser
Isolated:   no
Forward:    127.0.0.1:4000:40/tcp
"
        );
    }

    #[tokio::test]
    async fn test_show_all_fields() {
        let system = Arc::new(SystemMock::new());
        let env = Environment::new(
            UserName::from_str("cubic").unwrap(),
            String::new(),
            String::new(),
        );
        let instance_store = InstanceStoreMock::new(vec![Instance {
            name: "test".to_string(),
            arch: Arch::ARM64,
            user: UserName::from_str("john").unwrap(),
            cpus: 2,
            mem: DataSize::new(1),
            disk_capacity: DataSize::new(1),
            ssh_port: 8000,
            monitor_port: Some(8001),
            console_port: Some(8002),
            hostfwd: vec![
                "127.0.0.1:4000:40/tcp".parse().unwrap(),
                "0.0.0.0:80:8000/udp".parse().unwrap(),
            ],
            snapshots: vec![
                Snapshot {
                    name: "clean".to_string(),
                },
                Snapshot {
                    name: "before-upgrade".to_string(),
                },
            ],
            isolate: true,
            ssh_host_key: Some(
                "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIAABAgMEBQYHCAkKCwwNDg8QERITFBUWFxgZGhscHR4f"
                    .to_string(),
            ),
            ..Instance::default()
        }]);
        let context = commands::Context::new(
            Arc::new(SystemMock::new()),
            Console::new(Arc::clone(&system) as Arc<dyn System>),
            env,
            Box::new(instance_store),
        );

        let instance_dir = PathBuf::from("machines").join("test");
        let disk_image = instance_dir
            .join("machine.img")
            .to_string_lossy()
            .into_owned();
        let config = instance_dir
            .join("instance.toml")
            .to_string_lossy()
            .into_owned();
        let ssh_key = instance_dir
            .join("ssh_client_key")
            .to_string_lossy()
            .into_owned();

        ShowInstanceCommand {
            instance: InstanceName::from_str("test").unwrap().into(),
            all: true.into(),
        }
        .run(&context)
        .await
        .unwrap();

        assert_eq!(
            system.get_output(),
            format!(
                "\
Running:      no
Arch:         arm64
vCPUs:        2
Memory:       1 B
Disk Total:   1 B
User:         john
Isolated:     yes
Forward:      127.0.0.1:4000:40/tcp
              0.0.0.0:80:8000/udp
Snapshots:    clean
              before-upgrade
SSH Port:     8000
Monitor Port: 8001
Console Port: 8002
Disk Image:   {disk_image}
Config:       {config}
SSH Key:      {ssh_key}
SSH Host Key: SHA256:ZkAslGjFiUHdGf/WUL8rQvkib4PTvQatUV0OUQSncCA
SSH:          ssh -i {ssh_key} -p 8000 john@localhost
"
            )
        );
    }

    #[tokio::test]
    async fn test_show_command_failed() {
        let system = Arc::new(SystemMock::new());
        let env = Environment::new(
            UserName::from_str("testuser").unwrap(),
            String::new(),
            String::new(),
        );
        let instance_store = InstanceStoreMock::new(Vec::new());
        let context = commands::Context::new(
            Arc::new(SystemMock::new()),
            Console::new(Arc::clone(&system) as Arc<dyn System>),
            env,
            Box::new(instance_store),
        );

        assert!(matches!(
            ShowInstanceCommand {
                instance: InstanceName::from_str("test").unwrap().into(),
                all: false.into(),
            }
            .run(&context)
            .await,
            Err(Error::UnknownInstance(_))
        ));
    }
}
