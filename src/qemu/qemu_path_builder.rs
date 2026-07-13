use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[cfg(unix)]
const DEFAULT_DIRS: &[&str] = &[
    "/usr/bin",
    "/usr/local/bin",                 // Homebrew (Intel macOS)
    "/opt/homebrew/bin",              // Homebrew (Apple Silicon)
    "/home/linuxbrew/.linuxbrew/bin", // Homebrew (Linux)
    "/opt/local/bin",                 // MacPorts (native macOS)
];
#[cfg(windows)]
const DEFAULT_DIRS: &[&str] = &[r"C:\Program Files\qemu"];
#[cfg(not(any(windows, unix)))]
const DEFAULT_DIRS: &[&str] = &[];

pub struct QemuPathBuilder {
    dirs: Vec<PathBuf>,
}

impl QemuPathBuilder {
    pub fn new() -> Self {
        let mut dirs: Vec<PathBuf> = Vec::new();

        // Add QEMU directory override
        if let Some(dir) = std::env::var_os("CUBIC_QEMU_DIR") {
            dirs.push(PathBuf::from(dir));
        }

        // Add system PATH variable
        if let Some(path) = std::env::var_os("PATH") {
            dirs.extend(std::env::split_paths(&path));
        }

        // Add default paths
        dirs.extend(DEFAULT_DIRS.iter().map(PathBuf::from));

        Self::new_from_dirs(dirs)
    }

    pub fn new_from_dirs(mut dirs: Vec<PathBuf>) -> Self {
        // Preserve order, drop duplicates
        let mut seen = Vec::new();
        dirs.retain(|dir| {
            if seen.contains(dir) {
                false
            } else {
                seen.push(dir.clone());
                true
            }
        });

        Self { dirs }
    }

    pub fn build(&self) -> OsString {
        std::env::join_paths(&self.dirs).unwrap_or_default()
    }

    pub fn get_dirs(&self) -> &[PathBuf] {
        &self.dirs
    }
}

impl Default for QemuPathBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub fn find_in_dir(dir: &Path, name: &str) -> Option<PathBuf> {
    let candidate = dir.join(name);
    if candidate.exists() {
        return Some(candidate);
    }
    #[cfg(windows)]
    {
        let exe = dir.join(format!("{name}.exe"));
        if exe.exists() {
            return Some(exe);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_from_dirs_keeps_order_and_drops_duplicates() {
        let builder = QemuPathBuilder::new_from_dirs(
            ["/a", "/b", "/a", "/c", "/b"].map(PathBuf::from).to_vec(),
        );

        assert_eq!(builder.get_dirs(), ["/a", "/b", "/c"].map(PathBuf::from));
    }

    #[test]
    fn test_build_round_trips_through_path() {
        let builder =
            QemuPathBuilder::new_from_dirs(["/a", "/b", "/c"].map(PathBuf::from).to_vec());

        let joined = builder.build();

        let split: Vec<PathBuf> = std::env::split_paths(&joined).collect();
        assert_eq!(split, builder.get_dirs());
    }
}
