use crate::error::{Error, Result};
use crate::models::{Environment, Instance};
use crate::platform::System;
use crate::qemu::QemuPathBuilder;
use crate::util::SystemCommand;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageInfo {
    #[serde(alias = "actual-size")]
    pub actual_size: u64,
    #[serde(alias = "virtual-size")]
    pub virtual_size: u64,
}

pub struct QemuImg<'a> {
    system: &'a dyn System,
}

impl<'a> QemuImg<'a> {
    pub fn new(system: &'a dyn System) -> Self {
        Self { system }
    }

    fn command(&self) -> SystemCommand {
        let mut cmd = SystemCommand::new("qemu-img");
        cmd.set_env("PATH", QemuPathBuilder::new(self.system).build());
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
            .run_command(&command)
            .ok()
            .and_then(|stdout| String::from_utf8(stdout).ok())
            .and_then(|stdout| serde_json::from_str(&stdout).ok())
    }

    pub fn convert(&self, src: &str, dst: &str) -> Result<()> {
        let mut command = self.command();
        command
            .arg("convert")
            .arg("-f")
            .arg("qcow2")
            .arg("-O")
            .arg("qcow2")
            .arg(src)
            .arg(dst);

        self.system
            .run_command(&command)
            .map(|_| ())
            .map_err(Self::map_error)
    }

    pub fn resize(&self, image: &str, size: u64) -> Result<()> {
        let mut command = self.command();
        command.arg("resize").arg(image).arg(size.to_string());

        self.system
            .run_command(&command)
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
            "/run".to_string(),
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
    fn test_convert_reports_the_failure_of_qemu_img() {
        let system = SystemMock::new().add_failing_command(
            "qemu-img convert -f qcow2 -O qcow2 /cache/image /data/machines/test/image",
            "boom",
        );

        assert!(matches!(
            QemuImg::new(&system).convert("/cache/image", "/data/machines/test/image"),
            Err(Error::SystemCommandFailed(_, stderr)) if stderr == "boom"
        ));
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
        "dirty-flag": false
        }
        "#;

        let result: ImageInfo = serde_json::from_str(input).unwrap();
        assert_eq!(result.actual_size, 200704);
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
