use crate::commands::{self, Command};
use crate::error::{Error, Result};
use crate::util;
use crate::view::{Console, MapView};
use clap::Parser;

/// Show VM instances
#[derive(Parser)]
pub struct ShowInstanceCommand {
    #[clap(flatten)]
    pub instance: commands::InstanceArg,

    /// Show all available information
    #[arg(short = 'a', long = "all")]
    pub all: bool,
}

impl Command for ShowInstanceCommand {
    fn run(&self, console: &mut dyn Console, context: &commands::Context) -> Result<()> {
        let env = context.get_env();
        let instance_store = context.get_instance_store();

        if !instance_store.exists(self.instance.value.as_str()) {
            return Err(Error::UnknownInstance(self.instance.value.to_string()));
        }

        let instance = instance_store.load(self.instance.value.as_str())?;
        let ssh_key = env.get_ssh_private_key_file(&instance.name);

        let mut view = MapView::new();
        view.add(
            "Running",
            util::to_yes_no(instance_store.is_running(&instance)),
        );
        view.add(
            "PID",
            &instance_store
                .get_pid(&instance)
                .map(|pid| pid.to_string())
                .unwrap_or("n/a".to_string()),
        );
        view.add("Arch", &instance.arch.to_string());
        view.add("CPUs", &instance.cpus.to_string());
        view.add("Memory", &instance.mem.to_size());
        view.add(
            "Disk Used",
            &instance
                .disk_used
                .map(|size| size.to_size())
                .unwrap_or("n/a".to_string()),
        );
        view.add("Disk Total", &instance.disk_capacity.to_size());
        view.add("User", &instance.user);
        view.add("Isolated", util::to_yes_no(instance.isolate));
        view.add("SSH Port", &instance.ssh_port.to_string());
        view.add(
            "Monitor Port",
            &instance
                .monitor_port
                .map(|port| port.to_string())
                .unwrap_or("n/a".to_string()),
        );
        view.add(
            "Console Port",
            &instance
                .console_port
                .map(|port| port.to_string())
                .unwrap_or("n/a".to_string()),
        );

        // Port forwarding
        for (index, rule) in instance.hostfwd.iter().enumerate() {
            let key = if index == 0 { "Forward" } else { "" };
            view.add(key, &rule.to_string());
        }

        if self.all {
            view.add("Disk Image", &env.get_instance_image_file(&instance.name));
            view.add("Config", &env.get_instance_toml_config_file(&instance.name));
            view.add("SSH Key", &ssh_key);
            view.add(
                "SSH",
                &format!(
                    "ssh -i {} -p {} {}@localhost",
                    ssh_key, instance.ssh_port, instance.user
                ),
            );
        }

        view.print(console);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::ImageStoreMock;
    use crate::instance::InstanceStoreMock;
    use crate::models::{Arch, DataSize, Environment, Instance, InstanceName};
    use crate::view::ConsoleMock;
    use std::str::FromStr;

    #[test]
    fn test_show_command1() {
        let console = &mut ConsoleMock::new();
        let env = Environment::new(
            "myuser".to_string(),
            String::new(),
            String::new(),
            String::new(),
        );
        let image_store = ImageStoreMock::default();
        let instance_store = InstanceStoreMock::new(vec![Instance {
            name: "test".to_string(),
            arch: Arch::AMD64,
            user: "myuser".to_string(),
            cpus: 1,
            mem: DataSize::new(1024),
            disk_capacity: DataSize::new(1048576),
            ssh_port: 9000,
            hostfwd: vec!["127.0.0.1:4000:40/tcp".parse().unwrap()],
            ..Instance::default()
        }]);
        let context = commands::Context::new(env, Box::new(image_store), Box::new(instance_store));

        ShowInstanceCommand {
            instance: InstanceName::from_str("test").unwrap().into(),
            all: false,
        }
        .run(console, &context)
        .unwrap();

        assert_eq!(
            console.get_output(),
            "\
Running:      no
PID:          n/a
Arch:         amd64
CPUs:         1
Memory:       1.0 KiB
Disk Used:    n/a
Disk Total:   1.0 MiB
User:         myuser
Isolated:     no
SSH Port:     9000
Monitor Port: n/a
Console Port: n/a
Forward:      127.0.0.1:4000:40/tcp
"
        );
    }

    #[test]
    fn test_show_command2() {
        let console = &mut ConsoleMock::new();
        let env = Environment::new(
            "cubic".to_string(),
            String::new(),
            String::new(),
            String::new(),
        );
        let image_store = ImageStoreMock::default();
        let instance_store = InstanceStoreMock::new(vec![Instance {
            name: "test".to_string(),
            arch: Arch::ARM64,
            user: "john".to_string(),
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
            isolate: true,
            ..Instance::default()
        }]);
        let context = commands::Context::new(env, Box::new(image_store), Box::new(instance_store));

        ShowInstanceCommand {
            instance: InstanceName::from_str("test").unwrap().into(),
            all: true,
        }
        .run(console, &context)
        .unwrap();

        assert_eq!(
            console.get_output(),
            "\
Running:      no
PID:          n/a
Arch:         arm64
CPUs:         2
Memory:       1   B
Disk Used:    n/a
Disk Total:   1   B
User:         john
Isolated:     yes
SSH Port:     8000
Monitor Port: 8001
Console Port: 8002
Forward:      127.0.0.1:4000:40/tcp
              0.0.0.0:80:8000/udp
Disk Image:   machines/test/machine.img
Config:       machines/test/instance.toml
SSH Key:      machines/test/ssh_client_key
SSH:          ssh -i machines/test/ssh_client_key -p 8000 john@localhost
"
        );
    }

    #[test]
    fn test_show_command_failed() {
        let console = &mut ConsoleMock::new();
        let env = Environment::new(
            "testuser".to_string(),
            String::new(),
            String::new(),
            String::new(),
        );
        let instance_store = InstanceStoreMock::new(Vec::new());
        let image_store = ImageStoreMock::default();
        let context = commands::Context::new(env, Box::new(image_store), Box::new(instance_store));

        assert!(matches!(
            ShowInstanceCommand {
                instance: InstanceName::from_str("test").unwrap().into(),
                all: false,
            }
            .run(console, &context),
            Err(Error::UnknownInstance(_))
        ));
    }
}
