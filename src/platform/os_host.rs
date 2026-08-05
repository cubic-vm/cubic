use crate::platform::{Host, OsSystem};

impl Host for OsSystem {
    fn read_env_var(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }

    fn get_total_memory(&self) -> u64 {
        let mut system = sysinfo::System::new();
        system.refresh_memory();
        system.total_memory()
    }

    fn get_available_memory(&self) -> u64 {
        let mut system = sysinfo::System::new();
        system.refresh_memory();
        system.available_memory()
    }

    // Counts logical processors, so a host with simultaneous multithreading
    // reports its thread count rather than its physical cores.
    fn get_cpu_count(&self) -> u16 {
        let mut system = sysinfo::System::new();
        system.refresh_cpu_all();
        system.cpus().len() as u16
    }
}
