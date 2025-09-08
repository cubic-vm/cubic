use crate::env::Environment;
use crate::instance::Instance;
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

    pub fn get_image_info(&self, env: &Environment, instance: &Instance) -> Option<ImageInfo> {
        SystemCommand::new("qemu-img")
            .arg("info")
            .arg("--output")
            .arg("json")
            .arg(env.get_instance_image_file(&instance.name))
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .and_then(|stdout| serde_json::from_str(&stdout).ok())
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

        println!("{input}");
        let result: ImageInfo = serde_json::from_str(input).unwrap();
        assert_eq!(result.actual_size, 200704);
    }
}
