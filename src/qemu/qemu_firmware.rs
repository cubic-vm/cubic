use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::models::Arch;
use crate::platform::System;
use crate::qemu::qemu_firmware_descriptor::QemuFirmwareDescriptor;
use crate::qemu::qemu_path_builder::find_in_dir;

pub struct QemuFirmware;

impl QemuFirmware {
    pub fn locate(system: &dyn System, dirs: &[PathBuf], arch: Arch) -> Option<PathBuf> {
        let var = format!("CUBIC_QEMU_FW_{}", arch.as_vendor_str().to_uppercase());
        if let Some(fw) = system.read_env_var(&var) {
            return Some(PathBuf::from(fw)); // trust the override as-is
        }
        QemuInstall::find(dirs)?.find_firmware(system, arch)
    }
}

pub struct QemuInstall {
    prefix: PathBuf,
}

impl QemuInstall {
    pub fn find(dirs: &[PathBuf]) -> Option<Self> {
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

    pub fn get_prefix(&self) -> &Path {
        &self.prefix
    }

    pub fn find_module_dir(&self) -> Option<PathBuf> {
        self.build_module_dir_candidates()
            .into_iter()
            .find(|dir| Self::contains_shared_object(dir))
    }

    pub fn find_datadir(&self) -> Option<PathBuf> {
        let dir = self.prefix.join("share/qemu");
        dir.is_dir().then_some(dir)
    }

    fn build_module_dir_candidates(&self) -> Vec<PathBuf> {
        let mut candidates = Vec::new();
        if let Some(triplet) = self.find_lib_triplet() {
            candidates.push(self.prefix.join("lib").join(triplet).join("qemu"));
        }
        candidates.push(self.prefix.join("lib/qemu"));
        candidates.push(self.prefix.join("lib64/qemu"));
        candidates
    }

    fn find_lib_triplet(&self) -> Option<PathBuf> {
        let entries = std::fs::read_dir(self.prefix.join("lib")).ok()?;
        entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.join("qemu").is_dir())
            .find_map(|path| path.file_name().map(PathBuf::from))
    }

    fn contains_shared_object(dir: &Path) -> bool {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return false;
        };
        entries
            .flatten()
            .any(|entry| entry.path().extension().is_some_and(|ext| ext == "so"))
    }

    fn find_firmware(&self, system: &dyn System, arch: Arch) -> Option<PathBuf> {
        self.collect_descriptors(system)
            .into_iter()
            .filter(|descriptor| descriptor.matches(arch))
            .map(|descriptor| descriptor.build_code_path(&self.prefix))
            .find(|code| code.exists())
    }

    fn collect_descriptors(&self, system: &dyn System) -> Vec<QemuFirmwareDescriptor> {
        let mut by_name: BTreeMap<String, PathBuf> = BTreeMap::new();
        for dir in self.build_descriptor_dirs(system) {
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

    fn build_descriptor_dirs(&self, system: &dyn System) -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        if let Some(config) = system.read_env_var("XDG_CONFIG_HOME") {
            dirs.push(PathBuf::from(config).join("qemu/firmware"));
        } else if let Some(home) = system.read_env_var("HOME") {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::SystemMock;

    #[test]
    fn test_module_dir_candidates_cover_lib_layouts() {
        let candidates = QemuInstall {
            prefix: PathBuf::from("/snap/cubic/current/usr"),
        }
        .build_module_dir_candidates();
        assert_eq!(
            candidates,
            vec![
                PathBuf::from("/snap/cubic/current/usr/lib/qemu"),
                PathBuf::from("/snap/cubic/current/usr/lib64/qemu"),
            ]
        );
    }

    #[test]
    fn test_locate_trusts_override_env_var_without_scanning_dirs() {
        let system = SystemMock::new().add_env_var("CUBIC_QEMU_FW_AMD64", "/custom/fw.bin");

        let firmware = QemuFirmware::locate(&system, &[], Arch::AMD64);

        assert_eq!(firmware, Some(PathBuf::from("/custom/fw.bin")));
    }

    #[test]
    fn test_locate_returns_none_when_no_override_and_no_install_found() {
        let system = SystemMock::new();

        let firmware = QemuFirmware::locate(&system, &[], Arch::AMD64);

        assert_eq!(firmware, None);
    }

    #[test]
    fn test_build_descriptor_dirs_prefers_xdg_config_home_over_home() {
        let system = SystemMock::new()
            .add_env_var("XDG_CONFIG_HOME", "/xdg")
            .add_env_var("HOME", "/home/user");
        let install = QemuInstall {
            prefix: PathBuf::from("/prefix"),
        };

        let dirs = install.build_descriptor_dirs(&system);

        assert_eq!(dirs.first(), Some(&PathBuf::from("/xdg/qemu/firmware")));
    }

    #[test]
    fn test_build_descriptor_dirs_falls_back_to_home() {
        let system = SystemMock::new().add_env_var("HOME", "/home/user");
        let install = QemuInstall {
            prefix: PathBuf::from("/prefix"),
        };

        let dirs = install.build_descriptor_dirs(&system);

        assert_eq!(
            dirs.first(),
            Some(&PathBuf::from("/home/user/.config/qemu/firmware"))
        );
    }
}
