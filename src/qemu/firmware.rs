use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::models::Arch;

pub struct FirmwareFinder {
    arch: Arch,
    snap: Option<String>,
}

impl FirmwareFinder {
    pub fn new(arch: Arch, snap: Option<&str>) -> Self {
        Self {
            arch,
            snap: snap.map(str::to_owned),
        }
    }

    pub fn find(&self) -> Result<PathBuf> {
        if let Ok(fw) = std::env::var("CUBIC_FW") {
            return Ok(PathBuf::from(fw));
        }
        let share = self.qemu_share_dir();
        let err = match self.arch {
            Arch::AMD64 => Error::OvmfNotFound,
            Arch::ARM64 => Error::ArmFirmwareNotFound,
        };
        self.candidates(share.as_deref())
            .into_iter()
            .find(|p| p.exists())
            .ok_or(err)
    }

    fn qemu_share_dir(&self) -> Option<PathBuf> {
        let binary = match self.arch {
            Arch::AMD64 => "qemu-system-x86_64",
            Arch::ARM64 => "qemu-system-aarch64",
        };
        find_on_path(binary)
            .as_deref()
            .and_then(Path::parent) // bin/
            .and_then(Path::parent) // install root
            .map(|root| root.join("share").join("qemu"))
    }

    fn candidates(&self, qemu_share: Option<&Path>) -> Vec<PathBuf> {
        match self.arch {
            Arch::AMD64 => self.ovmf_candidates(qemu_share),
            Arch::ARM64 => self.arm_candidates(qemu_share),
        }
    }

    fn ovmf_candidates(&self, qemu_share: Option<&Path>) -> Vec<PathBuf> {
        let mut candidates = Vec::new();

        if let Some(snap_dir) = &self.snap {
            candidates.push(PathBuf::from(format!(
                "{snap_dir}/usr/share/OVMF/OVMF_CODE_4M.fd"
            )));
        }

        if let Some(share) = qemu_share {
            candidates.push(share.join("edk2-x86_64-code.fd"));
            candidates.push(share.join("OVMF_CODE_4M.fd"));
        }

        candidates.extend([
            // Debian/Ubuntu (ovmf)
            PathBuf::from("/usr/share/OVMF/OVMF_CODE_4M.fd"),
            // Fedora/RHEL (edk2-ovmf)
            PathBuf::from("/usr/share/edk2/ovmf/OVMF_CODE.4m.fd"),
            // Fedora older
            PathBuf::from("/usr/share/edk2-ovmf/OVMF_CODE.fd"),
            // Arch Linux (edk2-ovmf)
            PathBuf::from("/usr/share/edk2/x64/OVMF_CODE.4m.fd"),
            // openSUSE (qemu-ovmf-x86_64)
            PathBuf::from("/usr/share/qemu/ovmf-x86_64-code.bin"),
            // Homebrew Apple Silicon
            PathBuf::from("/opt/homebrew/share/qemu/edk2-x86_64-code.fd"),
            // Homebrew Intel / Linux Homebrew root
            PathBuf::from("/usr/local/share/qemu/edk2-x86_64-code.fd"),
            // Linux Homebrew non-root
            PathBuf::from("/home/linuxbrew/.linuxbrew/share/qemu/edk2-x86_64-code.fd"),
            // Windows QEMU installer
            PathBuf::from("C:/Program Files/QEMU/share/edk2-x86_64-code.fd"),
        ]);

        candidates
    }

    fn arm_candidates(&self, qemu_share: Option<&Path>) -> Vec<PathBuf> {
        let mut candidates = Vec::new();

        if let Some(snap_dir) = &self.snap {
            candidates.push(PathBuf::from(format!(
                "{snap_dir}/usr/share/AAVMF/AAVMF_CODE.fd"
            )));
        }

        if let Some(share) = qemu_share {
            candidates.push(share.join("edk2-aarch64-code.fd"));
        }

        candidates.extend([
            // Debian/Ubuntu (qemu-efi-aarch64 or ovmf) - 64 MB, pflash-safe
            PathBuf::from("/usr/share/AAVMF/AAVMF_CODE.fd"),
            // Fedora/RHEL (edk2-aarch64) - pflash-safe variant
            PathBuf::from("/usr/share/edk2/aarch64/QEMU_EFI-pflash.raw"),
            // Arch Linux (edk2-armvirt) - 64 MB on Arch
            PathBuf::from("/usr/share/edk2-armvirt/aarch64/QEMU_EFI.fd"),
            // openSUSE (qemu-uefi-aarch64)
            PathBuf::from("/usr/share/qemu/aavmf-aarch64-code.bin"),
            // Homebrew Apple Silicon
            PathBuf::from("/opt/homebrew/share/qemu/edk2-aarch64-code.fd"),
            // Homebrew Intel / Linux Homebrew root
            PathBuf::from("/usr/local/share/qemu/edk2-aarch64-code.fd"),
            // Linux Homebrew non-root
            PathBuf::from("/home/linuxbrew/.linuxbrew/share/qemu/edk2-aarch64-code.fd"),
            // Windows QEMU installer
            PathBuf::from("C:/Program Files/QEMU/share/edk2-aarch64-code.fd"),
        ]);

        candidates
    }
}

fn find_on_path(binary: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(binary);
        if candidate.exists() {
            return Some(candidate);
        }
        #[cfg(windows)]
        {
            let candidate = dir.join(format!("{binary}.exe"));
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ovmf_candidates_snap_is_first() {
        let candidates =
            FirmwareFinder::new(Arch::AMD64, Some("/snap/cubic/current")).candidates(None);
        assert_eq!(
            candidates[0],
            PathBuf::from("/snap/cubic/current/usr/share/OVMF/OVMF_CODE_4M.fd")
        );
    }

    #[test]
    fn test_ovmf_candidates_qemu_share_before_system() {
        let share = PathBuf::from("/opt/homebrew/share/qemu");
        let candidates = FirmwareFinder::new(Arch::AMD64, None).candidates(Some(&share));
        let qemu_pos = candidates
            .iter()
            .position(|p| p.starts_with("/opt/homebrew/share/qemu"))
            .unwrap();
        let debian_pos = candidates
            .iter()
            .position(|p| *p == PathBuf::from("/usr/share/OVMF/OVMF_CODE_4M.fd"))
            .unwrap();
        assert!(qemu_pos < debian_pos);
    }

    #[test]
    fn test_ovmf_candidates_all_platforms_present() {
        let candidates = FirmwareFinder::new(Arch::AMD64, None).candidates(None);
        let paths: Vec<&str> = candidates.iter().filter_map(|p| p.to_str()).collect();
        assert!(
            paths.iter().any(|p| p.contains("/usr/share/OVMF/")),
            "Debian/Ubuntu missing"
        );
        assert!(
            paths.iter().any(|p| p.contains("/usr/share/edk2/")),
            "Fedora missing"
        );
        assert!(
            paths.iter().any(|p| p.contains("/usr/share/edk2/x64/")),
            "Arch missing"
        );
        assert!(
            paths.iter().any(|p| p.contains("/usr/share/qemu/ovmf")),
            "openSUSE missing"
        );
        assert!(
            paths.iter().any(|p| p.contains("/opt/homebrew/")),
            "Homebrew Apple Silicon missing"
        );
        assert!(
            paths.iter().any(|p| p.contains("/usr/local/")),
            "Homebrew Intel missing"
        );
        assert!(
            paths.iter().any(|p| p.contains("linuxbrew")),
            "Linux Homebrew non-root missing"
        );
        assert!(
            paths.iter().any(|p| p.contains("Program Files/QEMU")),
            "Windows missing"
        );
    }

    #[test]
    fn test_arm_candidates_snap_is_first() {
        let candidates =
            FirmwareFinder::new(Arch::ARM64, Some("/snap/cubic/current")).candidates(None);
        assert_eq!(
            candidates[0],
            PathBuf::from("/snap/cubic/current/usr/share/AAVMF/AAVMF_CODE.fd")
        );
    }

    #[test]
    fn test_arm_candidates_all_platforms_present() {
        let candidates = FirmwareFinder::new(Arch::ARM64, None).candidates(None);
        let paths: Vec<&str> = candidates.iter().filter_map(|p| p.to_str()).collect();
        assert!(
            paths.iter().any(|p| p.contains("AAVMF")),
            "Debian/Ubuntu missing"
        );
        assert!(
            paths.iter().any(|p| p.contains("/usr/share/edk2/aarch64")),
            "Fedora missing"
        );
        assert!(
            paths.iter().any(|p| p.contains("edk2-armvirt")),
            "Arch missing"
        );
        assert!(
            paths.iter().any(|p| p.contains("/opt/homebrew/")),
            "Homebrew Apple Silicon missing"
        );
        assert!(
            paths.iter().any(|p| p.contains("/usr/local/")),
            "Homebrew Intel missing"
        );
        assert!(
            paths.iter().any(|p| p.contains("linuxbrew")),
            "Linux Homebrew non-root missing"
        );
        assert!(
            paths.iter().any(|p| p.contains("Program Files/QEMU")),
            "Windows missing"
        );
    }
}
