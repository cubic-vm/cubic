use crate::error::{Error, Result};
use crate::models::{Environment, Instance};
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

pub struct QemuImg;

impl QemuImg {
    pub fn new() -> Self {
        Self {}
    }

    fn command() -> SystemCommand {
        let mut cmd = SystemCommand::new("qemu-img");
        cmd.set_env("PATH", QemuPathBuilder::new().build());
        cmd
    }

    fn map_error(error: Error) -> Error {
        match error {
            Error::SystemCommandNotFound(_) => Error::QemuNotFound,
            other => other,
        }
    }

    pub fn get_image_info(&self, env: &Environment, instance: &Instance) -> Option<ImageInfo> {
        Self::command()
            .arg("info")
            .arg("--force-share")
            .arg("--output")
            .arg("json")
            .arg(env.get_instance_image_file(&instance.name))
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .and_then(|stdout| serde_json::from_str(&stdout).ok())
    }

    pub fn convert(&self, src: &str, dst: &str) -> Result<()> {
        Self::command()
            .arg("convert")
            .arg("-f")
            .arg("qcow2")
            .arg("-O")
            .arg("qcow2")
            .arg(src)
            .arg(dst)
            .run()
            .map_err(Self::map_error)
    }

    pub fn resize(&self, image: &str, size: u64) -> Result<()> {
        Self::command()
            .arg("resize")
            .arg(image)
            .arg(size.to_string())
            .run()
            .map_err(Self::map_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
