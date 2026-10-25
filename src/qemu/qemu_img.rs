use crate::error::{Error, Result};
use crate::models::{DataSize, Environment, Instance, Snapshot};
use crate::platform::System;
use crate::qemu::QemuPathBuilder;
use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Deserialize)]
pub struct ImageInfo {
    #[serde(alias = "actual-size")]
    pub actual_size: u64,
    #[serde(alias = "virtual-size")]
    pub virtual_size: u64,
    // Absent from the JSON until the image holds its first snapshot.
    #[serde(default)]
    pub snapshots: Vec<Snapshot>,
}

pub struct QemuImg<'a> {
    system: &'a dyn System,
}

impl<'a> QemuImg<'a> {
    pub fn new(system: &'a dyn System) -> Self {
        Self { system }
    }

    fn command(&self) -> Command {
        let mut cmd = Command::new("qemu-img");
        cmd.env("PATH", QemuPathBuilder::new(self.system).build());
        cmd
    }

    fn map_error(error: Error) -> Error {
        match error {
            Error::SystemCommandNotFound(_) => Error::QemuNotFound,
            other => other,
        }
    }

    // Anything that keeps the info from arriving, from a host without qemu to
    // an image the tool cannot read, leaves the caller with what it already
    // knows about the disk.
    pub fn get_image_info(&self, env: &Environment, instance: &Instance) -> Option<ImageInfo> {
        let mut command = self.command();
        command
            .arg("info")
            .arg("--force-share")
            .arg("--output")
            .arg("json")
            .arg(env.get_instance_image_file(&instance.name));

        self.system
            .run_and_wait(command, None, None)
            .ok()
            .and_then(|stdout| String::from_utf8(stdout).ok())
            .and_then(|stdout| serde_json::from_str(&stdout).ok())
    }

    // Keeps the current sizes when the image cannot be read.
    pub fn read_disk_info(&self, env: &Environment, instance: &mut Instance) {
        if let Some(info) = self.get_image_info(env, instance) {
            instance.disk_used = Some(DataSize::new(info.actual_size as usize));
            instance.disk_capacity = DataSize::new(info.virtual_size as usize);
            instance.snapshots = info.snapshots;
        }
    }

    pub fn create_overlay(&self, base_image: &str, image: &str) -> Result<()> {
        let mut command = self.command();
        command
            .arg("create")
            .arg("-q")
            .arg("-f")
            .arg("qcow2")
            .arg("-F")
            .arg("qcow2")
            .arg("-b")
            .arg(base_image)
            .arg(image);

        self.system
            .run_and_wait(command, None, None)
            .map(|_| ())
            .map_err(Self::map_error)
    }

    pub fn resize(&self, image: &str, size: u64) -> Result<()> {
        let mut command = self.command();
        command.arg("resize").arg(image).arg(size.to_string());

        self.system
            .run_and_wait(command, None, None)
            .map(|_| ())
            .map_err(Self::map_error)
    }

    pub fn create_snapshot(&self, image: &str, name: &str) -> Result<()> {
        self.run_snapshot_command(image, name, "-c")
    }

    pub fn restore_snapshot(&self, image: &str, name: &str) -> Result<()> {
        self.run_snapshot_command(image, name, "-a")
    }

    pub fn delete_snapshot(&self, image: &str, name: &str) -> Result<()> {
        self.run_snapshot_command(image, name, "-d")
    }

    fn run_snapshot_command(&self, image: &str, name: &str, action: &str) -> Result<()> {
        let mut command = self.command();
        command.arg("snapshot").arg(action).arg(name).arg(image);

        self.system
            .run_and_wait(command, None, None)
            .map(|_| ())
            .map_err(Self::map_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::UserName;
    use crate::platform::SystemMock;
    use std::str::FromStr;

    fn build_env() -> Environment {
        Environment::new(
            UserName::from_str("cubic").unwrap(),
            "/data".to_string(),
            "/cache".to_string(),
        )
    }

    fn build_instance() -> Instance {
        Instance {
            name: "test".to_string(),
            ..Instance::default()
        }
    }

    fn build_info_command(env: &Environment) -> String {
        format!(
            "qemu-img info --force-share --output json {}",
            env.get_instance_image_file("test")
        )
    }

    #[test]
    fn test_get_image_info_reads_the_reported_sizes() {
        let env = build_env();
        let system = SystemMock::new().add_command_output(
            &build_info_command(&env),
            br#"{"virtual-size": 1073741824, "actual-size": 200704}"#,
        );

        let info = QemuImg::new(&system)
            .get_image_info(&env, &build_instance())
            .unwrap();

        assert_eq!(info.virtual_size, 1073741824);
        assert_eq!(info.actual_size, 200704);
    }

    #[test]
    fn test_get_image_info_is_none_without_qemu() {
        let env = build_env();
        let system = SystemMock::new();

        assert!(
            QemuImg::new(&system)
                .get_image_info(&env, &build_instance())
                .is_none()
        );
    }

    #[test]
    fn test_get_image_info_is_none_when_the_output_is_not_readable() {
        let env = build_env();
        let system = SystemMock::new().add_command_output(&build_info_command(&env), b"not json");

        assert!(
            QemuImg::new(&system)
                .get_image_info(&env, &build_instance())
                .is_none()
        );
    }

    #[test]
    fn test_resize_reports_a_missing_qemu() {
        let system = SystemMock::new();

        assert!(matches!(
            QemuImg::new(&system).resize("/data/machines/test/image", 2048),
            Err(Error::QemuNotFound)
        ));
    }

    #[test]
    fn test_create_overlay_backs_the_image_by_the_base_image() {
        let command = "qemu-img create -q -f qcow2 -F qcow2 -b /cache/images/debian /data/machines/test/machine.img";
        let system = SystemMock::new().add_command_output(command, b"");

        QemuImg::new(&system)
            .create_overlay("/cache/images/debian", "/data/machines/test/machine.img")
            .unwrap();

        assert_eq!(system.get_executed_commands(), vec![command]);
    }

    #[test]
    fn test_resize_passes_the_size_to_qemu_img() {
        let system = SystemMock::new()
            .add_command_output("qemu-img resize /data/machines/test/image 2048", b"");

        QemuImg::new(&system)
            .resize("/data/machines/test/image", 2048)
            .unwrap();

        assert_eq!(
            system.get_executed_commands(),
            vec!["qemu-img resize /data/machines/test/image 2048"]
        );
    }

    #[test]
    fn test_image_info() {
        let input = r#"
        {
        "virtual-size": 1073741824,
        "filename": "/tmp/cache/cubic/images/ubuntu_noble_amd64",
        "cluster-size": 65536,
        "format": "qcow2",
        "actual-size": 200704,
        "format-specific": {
            "type": "qcow2",
            "data": {
                "compat": "1.1",
                "compression-type": "zlib",
                "lazy-refcounts": false,
                "refcount-bits": 16,
                "corrupt": false,
                "extended-l2": false
            }
        },
        "dirty-flag": false,
        "snapshots": [
            {
                "icount": 0,
                "vm-clock-nsec": 0,
                "name": "clean",
                "date-sec": 1787430192,
                "date-nsec": 207919000,
                "vm-clock-sec": 0,
                "id": "1",
                "vm-state-size": 0
            }
        ]
        }
        "#;

        let result: ImageInfo = serde_json::from_str(input).unwrap();
        assert_eq!(result.actual_size, 200704);
        assert_eq!(result.snapshots.len(), 1);
        assert_eq!(result.snapshots[0].name, "clean");
    }

    #[test]
    fn test_snapshot_commands_pass_the_flag_and_name_to_qemu_img() {
        let image = "/data/machines/test/machine.img";
        let system = SystemMock::new()
            .add_command_output(&format!("qemu-img snapshot -c clean {image}"), b"")
            .add_command_output(&format!("qemu-img snapshot -a clean {image}"), b"")
            .add_command_output(&format!("qemu-img snapshot -d clean {image}"), b"");
        let qemu_img = QemuImg::new(&system);

        qemu_img.create_snapshot(image, "clean").unwrap();
        qemu_img.restore_snapshot(image, "clean").unwrap();
        qemu_img.delete_snapshot(image, "clean").unwrap();

        assert_eq!(
            system.get_executed_commands(),
            vec![
                format!("qemu-img snapshot -c clean {image}"),
                format!("qemu-img snapshot -a clean {image}"),
                format!("qemu-img snapshot -d clean {image}"),
            ]
        );
    }

    #[test]
    fn test_read_disk_info_reads_the_snapshots() {
        let env = build_env();
        let system = SystemMock::new().add_command_output(
            &build_info_command(&env),
            br#"{"virtual-size": 1073741824, "actual-size": 200704,
                 "snapshots": [{"name": "clean"}, {"name": "deps"}]}"#,
        );
        let mut instance = build_instance();

        QemuImg::new(&system).read_disk_info(&env, &mut instance);

        let names: Vec<&str> = instance
            .snapshots
            .iter()
            .map(|snapshot| snapshot.name.as_str())
            .collect();
        assert_eq!(names, vec!["clean", "deps"]);
    }

    #[test]
    fn test_map_error_translates_not_found() {
        assert!(matches!(
            QemuImg::map_error(Error::SystemCommandNotFound("qemu-img".to_string())),
            Error::QemuNotFound
        ));
    }

    #[test]
    fn test_map_error_passes_other_errors_through() {
        assert!(matches!(
            QemuImg::map_error(Error::SystemCommandFailed(
                "cmd".to_string(),
                "boom".to_string()
            )),
            Error::SystemCommandFailed(..)
        ));
    }
}
