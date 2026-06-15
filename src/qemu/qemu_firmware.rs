use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::models::Arch;
use crate::qemu::qemu_firmware_descriptor::QemuFirmwareDescriptor;
use crate::qemu::qemu_path_builder::find_in_dir;

pub struct QemuFirmware;

impl QemuFirmware {
    pub fn locate(dirs: &[PathBuf], arch: Arch) -> Option<PathBuf> {
        let var = format!("CUBIC_QEMU_FW_{}", arch.as_vendor_str().to_uppercase());
        if let Some(fw) = std::env::var_os(&var) {
            return Some(PathBuf::from(fw)); // trust the override as-is
        }
        QemuInstall::find(dirs)?.find_firmware(arch)
    }
}

struct QemuInstall {
    prefix: PathBuf,
}

impl QemuInstall {
    fn find(dirs: &[PathBuf]) -> Option<Self> {
        let names = ["qemu-system-x86_64", "qemu-system-aarch64"];
        let dir = dirs
            .iter()
            .find(|dir| names.iter().any(|name| find_in_dir(dir, name).is_some()))?;
        let prefix = if cfg!(windows) {
            dir.clone()
        } else {
            dir.parent().unwrap_or(dir).to_path_buf()
        };
        Some(Self { prefix })
    }

    fn find_firmware(&self, arch: Arch) -> Option<PathBuf> {
        self.collect_descriptors()
            .into_iter()
            .filter(|descriptor| descriptor.matches(arch))
            .map(|descriptor| descriptor.build_code_path(&self.prefix))
            .find(|code| code.exists())
    }

    fn collect_descriptors(&self) -> Vec<QemuFirmwareDescriptor> {
        let mut by_name: BTreeMap<String, PathBuf> = BTreeMap::new();
        for dir in self.build_descriptor_dirs() {
            for path in Self::find_json_files(&dir) {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    by_name.entry(name.to_owned()).or_insert(path);
                }
            }
        }
        by_name
            .values()
            .filter_map(|path| QemuFirmwareDescriptor::parse(&std::fs::read_to_string(path).ok()?))
            .collect()
    }

    fn build_descriptor_dirs(&self) -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        if let Some(config) = std::env::var_os("XDG_CONFIG_HOME") {
            dirs.push(PathBuf::from(config).join("qemu/firmware"));
        } else if let Some(home) = std::env::var_os("HOME") {
            dirs.push(PathBuf::from(home).join(".config/qemu/firmware"));
        }
        dirs.push(PathBuf::from("/etc/qemu/firmware"));
        dirs.push(self.prefix.join("share/qemu/firmware"));
        dirs.push(self.prefix.join("share/firmware"));
        dirs.push(self.prefix.join("firmware"));
        dirs
    }

    fn find_json_files(dir: &Path) -> Vec<PathBuf> {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return Vec::new();
        };
        entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
            .collect()
    }
}
