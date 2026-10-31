use crate::actions::LoadInstanceAction;
use crate::commands::{self, Command};
use crate::error::Result;
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

        let instance = LoadInstanceAction::new().run(context, self.instance.value.as_str())?;
        let ssh_key = env.get_ssh_private_key_file(&instance.name);

        let mut view = MapView::new();
        if let Some(os) = &instance.os {
            view.add("OS", os);
        }
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
            let image_file = env.get_instance_image_file(&instance.name);
            let config_file = env.get_instance_toml_config_file(&instance.name);
            view.add("SSH Port", &instance.ssh_port.to_string());
            view.add("Disk Image", &image_file.display().to_string());
            view.add("Config", &config_file.display().to_string());
            view.add("SSH Key", &ssh_key.display().to_string());
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
                    ssh_key.display(),
                    instance.ssh_port,
                    instance.user
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
    use crate::commands::ContextBuilder;
    use crate::models::{Arch, DataSize, Instance, InstanceName, Snapshot, UserName};
    use crate::platform::SystemMock;
    use std::str::FromStr;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_show_basic_fields() {
        let mut builder = ContextBuilder::new(SystemMock::new());
        builder.get_store().add_instances(vec![Instance {
            name: "test".to_string(),
            arch: Arch::AMD64,
            user: UserName::from_str("myuser").unwrap(),
            cpus: 1,
            mem: DataSize::new(1024),
            disk_capacity: DataSize::new(1048576),
            os: Some("ubuntu:24.04".to_string()),
            ssh_port: 9000,
            hostfwd: vec!["127.0.0.1:4000:40/tcp".parse().unwrap()],
            ..Instance::default()
        }]);
        let system = Arc::clone(builder.get_system());
        let context = builder.build();

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
OS:         ubuntu:24.04
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
        let mut builder = ContextBuilder::new(SystemMock::new());
        builder.get_store().add_instances(vec![Instance {
            name: "test".to_string(),
            arch: Arch::ARM64,
            user: UserName::from_str("john").unwrap(),
            cpus: 2,
            mem: DataSize::new(1),
            disk_capacity: DataSize::new(1),
            ssh_port: 8000,
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
        let system = Arc::clone(builder.get_system());
        let context = builder.build();

        let env = context.get_env();
        let disk_image = env.get_instance_image_file("test");
        let config = env.get_instance_toml_config_file("test");
        let ssh_key = env.get_ssh_private_key_file("test");

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
Disk Image:   {}
Config:       {}
SSH Key:      {}
SSH Host Key: SHA256:ZkAslGjFiUHdGf/WUL8rQvkib4PTvQatUV0OUQSncCA
SSH:          ssh -i {} -p 8000 john@localhost
",
                disk_image.display(),
                config.display(),
                ssh_key.display(),
                ssh_key.display()
            )
        );
    }
}
