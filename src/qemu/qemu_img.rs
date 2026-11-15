use crate::error::{Error, Result};
use crate::models::{DataSize, Snapshot};
use crate::platform::System;
use crate::qemu::QemuPathBuilder;
use crate::util::SystemCommand;
use serde::Deserialize;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Deserialize)]
pub struct ImageInfo {
    #[serde(alias = "actual-size")]
    pub actual_size: DataSize,
    #[serde(alias = "virtual-size")]
    pub virtual_size: DataSize,
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

    async fn run(&self, command: Command) -> Result<Vec<u8>> {
        SystemCommand::new(command)
            .run(self.system, None, None)
            .await
            .map_err(Self::map_error)
    }

    // Returns None when the info cannot be read, from a host without qemu to
    // an image the tool cannot parse.
    pub async fn get_image_info(&self, image: &Path) -> Option<ImageInfo> {
        let mut command = self.command();
        command
            .arg("info")
            .arg("--force-share")
            .arg("--output")
            .arg("json")
            .arg(image);

        self.run(command)
            .await
            .ok()
            .and_then(|stdout| String::from_utf8(stdout).ok())
            .and_then(|stdout| serde_json::from_str(&stdout).ok())
    }

    // An unreadable image holds no snapshots as far as the caller can tell.
    pub async fn has_snapshot(&self, image: &Path, name: &str) -> bool {
        self.get_image_info(image)
            .await
            .is_some_and(|info| info.snapshots.iter().any(|snapshot| snapshot.name == name))
    }

    pub async fn create_overlay(&self, base_image: &Path, image: &Path) -> Result<()> {
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

        self.run(command).await.map(|_| ())
    }

    pub async fn resize(&self, image: &Path, size: u64) -> Result<()> {
        let mut command = self.command();
        command.arg("resize").arg(image).arg(size.to_string());

        self.run(command).await.map(|_| ())
    }

    pub async fn create_snapshot(&self, image: &Path, name: &str) -> Result<()> {
        self.run_snapshot_command(image, name, "-c").await
    }

    pub async fn restore_snapshot(&self, image: &Path, name: &str) -> Result<()> {
        self.run_snapshot_command(image, name, "-a").await
    }

    pub async fn delete_snapshot(&self, image: &Path, name: &str) -> Result<()> {
        self.run_snapshot_command(image, name, "-d").await
    }

    async fn run_snapshot_command(&self, image: &Path, name: &str, action: &str) -> Result<()> {
        let mut command = self.command();
        command.arg("snapshot").arg(action).arg(name).arg(image);

        self.run(command).await.map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::SystemMock;

    const IMAGE: &str = "/data/machines/test/machine.img";

    fn build_info_command() -> String {
        format!("qemu-img info --force-share --output json {IMAGE}")
    }

    #[tokio::test]
    async fn test_get_image_info_reads_the_reported_sizes() {
        let system = SystemMock::new().add_command_output(
            &build_info_command(),
            br#"{"virtual-size": 1073741824, "actual-size": 200704}"#,
        );

        let info = QemuImg::new(&system)
            .get_image_info(Path::new(IMAGE))
            .await
            .unwrap();

        assert_eq!(info.virtual_size.get_bytes(), 1073741824);
        assert_eq!(info.actual_size.get_bytes(), 200704);
    }

    #[tokio::test]
    async fn test_get_image_info_is_none_without_qemu() {
        let system = SystemMock::new();

        assert!(
            QemuImg::new(&system)
                .get_image_info(Path::new(IMAGE))
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_get_image_info_is_none_when_the_output_is_not_readable() {
        let system = SystemMock::new().add_command_output(&build_info_command(), b"not json");

        assert!(
            QemuImg::new(&system)
                .get_image_info(Path::new(IMAGE))
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_resize_reports_a_missing_qemu() {
        let system = SystemMock::new();

        assert!(matches!(
            QemuImg::new(&system)
                .resize(Path::new("/data/machines/test/image"), 2048)
                .await,
            Err(Error::QemuNotFound)
        ));
    }

    #[tokio::test]
    async fn test_create_overlay_backs_the_image_by_the_base_image() {
        let command = "qemu-img create -q -f qcow2 -F qcow2 -b /cache/images/debian /data/machines/test/machine.img";
        let system = SystemMock::new().add_command_output(command, b"");

        QemuImg::new(&system)
            .create_overlay(
                Path::new("/cache/images/debian"),
                Path::new("/data/machines/test/machine.img"),
            )
            .await
            .unwrap();

        assert_eq!(system.get_executed_commands(), vec![command]);
    }

    #[tokio::test]
    async fn test_resize_passes_the_size_to_qemu_img() {
        let system = SystemMock::new()
            .add_command_output("qemu-img resize /data/machines/test/image 2048", b"");

        QemuImg::new(&system)
            .resize(Path::new("/data/machines/test/image"), 2048)
            .await
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
        assert_eq!(result.actual_size.get_bytes(), 200704);
        assert_eq!(result.snapshots.len(), 1);
        assert_eq!(result.snapshots[0].name, "clean");
    }

    #[tokio::test]
    async fn test_snapshot_commands_pass_the_flag_and_name_to_qemu_img() {
        let image = "/data/machines/test/machine.img";
        let system = SystemMock::new()
            .add_command_output(&format!("qemu-img snapshot -c clean {image}"), b"")
            .add_command_output(&format!("qemu-img snapshot -a clean {image}"), b"")
            .add_command_output(&format!("qemu-img snapshot -d clean {image}"), b"");
        let qemu_img = QemuImg::new(&system);

        qemu_img
            .create_snapshot(Path::new(image), "clean")
            .await
            .unwrap();
        qemu_img
            .restore_snapshot(Path::new(image), "clean")
            .await
            .unwrap();
        qemu_img
            .delete_snapshot(Path::new(image), "clean")
            .await
            .unwrap();

        assert_eq!(
            system.get_executed_commands(),
            vec![
                format!("qemu-img snapshot -c clean {image}"),
                format!("qemu-img snapshot -a clean {image}"),
                format!("qemu-img snapshot -d clean {image}"),
            ]
        );
    }

    #[tokio::test]
    async fn test_has_snapshot_reads_the_snapshots_of_the_image() {
        let system = SystemMock::new().add_command_output(
            &build_info_command(),
            br#"{"virtual-size": 1073741824, "actual-size": 200704,
                 "snapshots": [{"name": "clean"}, {"name": "deps"}]}"#,
        );
        let qemu_img = QemuImg::new(&system);

        assert!(qemu_img.has_snapshot(Path::new(IMAGE), "deps").await);
        assert!(!qemu_img.has_snapshot(Path::new(IMAGE), "other").await);
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
