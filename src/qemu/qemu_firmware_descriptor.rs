use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::models::Arch;

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct QemuFirmwareDescriptor {
    #[serde(default)]
    interface_types: Vec<String>,
    mapping: Mapping,
    #[serde(default)]
    targets: Vec<Target>,
    #[serde(default)]
    features: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Mapping {
    device: String,
    executable: FileRef,
}

#[derive(Deserialize)]
struct FileRef {
    filename: String,
}

#[derive(Deserialize)]
struct Target {
    architecture: String,
    #[serde(default)]
    machines: Vec<String>,
}

const EXCLUDED_FEATURES: &[&str] = &["secure-boot", "requires-smm", "amd-sev-snp", "intel-tdx"];

impl QemuFirmwareDescriptor {
    pub fn parse(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }

    pub fn matches(&self, arch: Arch) -> bool {
        let machine = match arch {
            Arch::AMD64 => "q35",
            Arch::ARM64 => "virt",
        };
        self.mapping.device == "flash"
            && self.interface_types.iter().any(|i| i == "uefi")
            && self
                .features
                .iter()
                .all(|f| !EXCLUDED_FEATURES.contains(&f.as_str()))
            && self.targets.iter().any(|target| {
                target.architecture == arch.as_canonical_str()
                    && target.machines.iter().any(|m| m.contains(machine))
            })
    }

    pub fn build_code_path(&self, prefix: &Path) -> PathBuf {
        let code = Path::new(&self.mapping.executable.filename);
        match code.components().position(|c| c.as_os_str() == "share") {
            Some(index) => prefix.join(code.components().skip(index).collect::<PathBuf>()),
            None => code.to_path_buf(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_descriptor(arch: &str, machine: &str, code: &str, features: &str) -> String {
        format!(
            r#"{{
              "description": "test",
              "interface-types": ["uefi"],
              "mapping": {{
                "device": "flash",
                "executable": {{ "filename": "{code}", "format": "raw" }},
                "nvram-template": {{ "filename": "/x/VARS.fd", "format": "raw" }}
              }},
              "targets": [{{ "architecture": "{arch}", "machines": ["{machine}"] }}],
              "features": [{features}],
              "tags": []
            }}"#
        )
    }

    #[test]
    fn test_parse_descriptor() {
        let json = build_descriptor("x86_64", "pc-q35-8.0", "/fw/OVMF_CODE.fd", "");
        let parsed = QemuFirmwareDescriptor::parse(&json).unwrap();
        assert_eq!(parsed.mapping.device, "flash");
        assert_eq!(parsed.mapping.executable.filename, "/fw/OVMF_CODE.fd");
        assert!(parsed.interface_types.iter().any(|i| i == "uefi"));
        assert_eq!(parsed.targets[0].architecture, "x86_64");
        assert_eq!(parsed.targets[0].machines[0], "pc-q35-8.0");
    }

    #[test]
    fn test_matches_plain_amd64() {
        let plain =
            QemuFirmwareDescriptor::parse(&build_descriptor("x86_64", "pc-q35-8.0", "/c", ""))
                .unwrap();
        assert!(plain.matches(Arch::AMD64));
        assert!(!plain.matches(Arch::ARM64));
    }

    #[test]
    fn test_matches_plain_descriptor_advertising_amd_sev() {
        let plain = QemuFirmwareDescriptor::parse(&build_descriptor(
            "x86_64",
            "pc-q35-8.0",
            "/c",
            "\"acpi-s3\", \"amd-sev\", \"amd-sev-es\", \"verbose-dynamic\"",
        ))
        .unwrap();
        assert!(plain.matches(Arch::AMD64));
    }

    #[test]
    fn test_matches_rejects_non_q35_bios_and_secure_boot() {
        let i440fx =
            QemuFirmwareDescriptor::parse(&build_descriptor("x86_64", "pc-i440fx-8.0", "/c", ""))
                .unwrap();
        assert!(!i440fx.matches(Arch::AMD64));

        let secure = QemuFirmwareDescriptor::parse(&build_descriptor(
            "x86_64",
            "pc-q35-8.0",
            "/c",
            "\"secure-boot\"",
        ))
        .unwrap();
        assert!(!secure.matches(Arch::AMD64));

        let bios = QemuFirmwareDescriptor::parse(
            r#"{ "interface-types": ["bios"],
                 "mapping": { "device": "flash", "executable": { "filename": "/c" } },
                 "targets": [{ "architecture": "x86_64", "machines": ["pc-q35-8.0"] }] }"#,
        )
        .unwrap();
        assert!(!bios.matches(Arch::AMD64));
    }

    #[test]
    fn test_matches_rejects_sev_and_tdx_specific() {
        for feature in ["amd-sev-snp", "intel-tdx", "requires-smm"] {
            let descriptor = QemuFirmwareDescriptor::parse(&build_descriptor(
                "x86_64",
                "pc-q35-8.0",
                "/c",
                &format!("\"{feature}\""),
            ))
            .unwrap();
            assert!(!descriptor.matches(Arch::AMD64), "{feature} should reject");
        }
    }

    fn build_descriptor_with_code(code: &str) -> QemuFirmwareDescriptor {
        QemuFirmwareDescriptor::parse(&build_descriptor("x86_64", "pc-q35-8.0", code, "")).unwrap()
    }

    #[test]
    fn test_build_code_path_maps_absolute_onto_prefix() {
        let prefix = Path::new("/snap/cubic/current/usr");
        assert_eq!(
            build_descriptor_with_code("/usr/share/OVMF/OVMF_CODE_4M.fd").build_code_path(prefix),
            PathBuf::from("/snap/cubic/current/usr/share/OVMF/OVMF_CODE_4M.fd")
        );
    }

    #[test]
    fn test_build_code_path_maps_relative_onto_prefix() {
        let prefix = Path::new("/opt/qemu");
        assert_eq!(
            build_descriptor_with_code("share/qemu/edk2-x86_64-code.fd").build_code_path(prefix),
            PathBuf::from("/opt/qemu/share/qemu/edk2-x86_64-code.fd")
        );
    }

    #[test]
    fn test_build_code_path_without_share_segment_is_identity() {
        let prefix = Path::new("/opt/qemu");
        assert_eq!(
            build_descriptor_with_code("/custom/OVMF_CODE.fd").build_code_path(prefix),
            PathBuf::from("/custom/OVMF_CODE.fd")
        );
    }
}
