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
    fn run(&self, console: &mut Console<'_>, context: &commands::Context) -> Result<()> {
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
            &util::format_or_na(instance_store.get_pid(&instance)),
        );
        view.add("Arch", &instance.arch.to_string());
        view.add("CPUs", &instance.cpus.to_string());
        view.add("Memory", &instance.mem.to_size());
        view.add(
            "Disk Used",
            &util::format_or_na(instance.disk_used.map(|size| size.to_size())),
        );
        view.add("Disk Total", &instance.disk_capacity.to_size());
        view.add("User", instance.user.as_str());
        view.add("Isolated", util::to_yes_no(instance.isolate));
        view.add("SSH Port", &instance.ssh_port.to_string());
        view.add("Monitor Port", &util::format_or_na(instance.monitor_port));
        view.add("Console Port", &util::format_or_na(instance.console_port));

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
    use crate::models::{Arch, DataSize, Environment, Instance, InstanceName, UserName};
    use crate::platform::SystemMock;
    use std::path::PathBuf;
    use std::rc::Rc;
    use std::str::FromStr;

    #[test]
    fn test_show_basic_fields() {
        let system = SystemMock::new();
        let console = &mut Console::new(&system);
        let env = Environment::new(
            UserName::from_str("myuser").unwrap(),
            String::new(),
            String::new(),
            String::new(),
        );
        let image_store = ImageStoreMock::default();
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
            Rc::new(SystemMock::new()),
            env,
            Box::new(image_store),
            Box::new(instance_store),
        );

        ShowInstanceCommand {
            instance: InstanceName::from_str("test").unwrap().into(),
            all: false,
        }
        .run(console, &context)
        .unwrap();

        assert_eq!(
            system.get_output(),
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
    fn test_show_all_fields() {
        let system = SystemMock::new();
        let console = &mut Console::new(&system);
        let env = Environment::new(
            UserName::from_str("cubic").unwrap(),
            String::new(),
            String::new(),
            String::new(),
        );
        let image_store = ImageStoreMock::default();
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
            isolate: true,
            ..Instance::default()
        }]);
        let context = commands::Context::new(
            Rc::new(SystemMock::new()),
            env,
            Box::new(image_store),
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
            all: true,
        }
        .run(console, &context)
        .unwrap();

        assert_eq!(
            system.get_output(),
            format!(
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
Disk Image:   {disk_image}
Config:       {config}
SSH Key:      {ssh_key}
SSH:          ssh -i {ssh_key} -p 8000 john@localhost
"
            )
        );
    }

    #[test]
    fn test_show_command_failed() {
        let system = SystemMock::new();
        let console = &mut Console::new(&system);
        let env = Environment::new(
            UserName::from_str("testuser").unwrap(),
            String::new(),
            String::new(),
            String::new(),
        );
        let instance_store = InstanceStoreMock::new(Vec::new());
        let image_store = ImageStoreMock::default();
        let context = commands::Context::new(
            Rc::new(SystemMock::new()),
            env,
            Box::new(image_store),
            Box::new(instance_store),
        );

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
