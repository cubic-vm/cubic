use crate::models::TimezoneName;
use crate::platform::{Host, OsSystem};
use std::path::PathBuf;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind};

impl Host for OsSystem {
    fn read_env_var(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }

    fn get_total_memory(&self) -> Option<u64> {
        let mut system = sysinfo::System::new().ok()?;
        system.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
        Some(system.total_memory())
    }

    fn get_available_memory(&self) -> Option<u64> {
        let mut system = sysinfo::System::new().ok()?;
        system.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
        Some(system.available_memory())
    }

    // Counts logical processors, matching how vCPUs map to host threads.
    fn get_cpu_count(&self) -> Option<u16> {
        let mut system = sysinfo::System::new().ok()?;
        system.refresh_cpu_list(CpuRefreshKind::nothing());
        Some(system.cpus().len() as u16)
    }

    fn get_timezone_name(&self) -> Option<TimezoneName> {
        iana_time_zone::get_timezone().ok()?.parse().ok()
    }

    fn get_current_exe(&self) -> PathBuf {
        std::env::current_exe().unwrap_or_else(|_| PathBuf::from("cubic"))
    }
}
