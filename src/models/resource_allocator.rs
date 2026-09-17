use crate::models::{DataSize, Environment};
use crate::platform::System;
use std::path::Path;

const MIB: usize = 1024 * 1024;
const GIB: usize = 1024 * MIB;

/// The smallest usable size for each resource. A value below this can never
/// produce a machine that boots.
pub const MIN_CPUS: u16 = 1;
pub const MIN_MEMORY: usize = 512 * MIB;
pub const MIN_DISK: usize = GIB;

/// Memory kept aside for the host and QEMU overhead so a machine never claims
/// all of the available memory.
pub const HOST_MEMORY_RESERVE: usize = GIB;

pub const LOW_DISK_SPACE: u64 = 5 * GIB as u64;
pub const LOW_DISK_SPACE_WARNING: &str = "Low disk space detected on the host.";

/// Decides the default vCPU count and memory for a new machine based on the
/// resources of the host it runs on.
///
/// A fixed default of 4 vCPUs and 4G is wasteful on small hosts. QEMU reserves
/// the full memory amount at startup, so handing one machine all the memory can
/// make the host unusable. This allocator picks a level from the host size where
/// level N means 2N vCPUs and N GiB of memory, so every level keeps half a GiB
/// of memory per vCPU.
///
/// A level requires the host to have at least 4 * (level + 1) threads and the
/// same number of GiB of memory, so the chosen level is the largest one both
/// totals allow. Because the machine grows linearly while the requirement grows
/// faster, the machine always stays under half the host threads and under a
/// quarter of the host memory, and the level keeps scaling with the host without
/// an upper bound. This keeps cubic machines conservative by default since they
/// are meant for lighter workloads and a user can always override the size. A
/// host too small for level 1 falls back to 1 vCPU and 512 MiB.
pub struct ResourceAllocator {
    host_mem_bytes: usize,
    host_threads: u16,
}

impl ResourceAllocator {
    /// Build an allocator from explicit host totals. Useful for testing without
    /// touching the real system. `host_threads` is the count of logical
    /// processors, including simultaneous multithreading siblings.
    pub fn new(host_mem_bytes: usize, host_threads: u16) -> Self {
        Self {
            host_mem_bytes,
            host_threads,
        }
    }

    /// Read the live host total memory and cpu count and build an allocator.
    ///
    /// The cpu count is the number of logical processors, so a host with
    /// simultaneous multithreading reports its thread count rather than its
    /// physical cores. This matches the vCPU count handed to a machine, which
    /// also maps to threads.
    pub fn read_from_host(system: &dyn System) -> Self {
        Self::new(system.get_total_memory() as usize, system.get_cpu_count())
    }

    /// Return the default vCPU count and memory for a new machine by selecting
    /// the highest level the host satisfies and falling back to 1 vCPU and
    /// 512 MiB when the host is too small for level 1.
    ///
    /// A level needs 4 * (level + 1) threads and the same number of GiB, so the
    /// largest level a total allows is that total divided by four minus one. The
    /// chosen level is the smaller of the two limits.
    pub fn get_default_resources(&self) -> (u16, DataSize) {
        let by_threads = self.host_threads as usize / 4;
        let by_memory = self.host_mem_bytes / (4 * GIB);
        Self::resources_for_level(by_threads.min(by_memory).saturating_sub(1))
    }

    /// Return the largest level whose memory fits in `available_bytes` after
    /// holding back the host reserve. Level N is 2N vCPUs and N GiB, so the
    /// level is the budget in whole GiB. Falls back to 1 vCPU and 512 MiB and
    /// returns None when not even that fits.
    pub fn get_resources_for_budget(available_bytes: usize) -> Option<(u16, DataSize)> {
        let budget = available_bytes.saturating_sub(HOST_MEMORY_RESERVE);
        (budget >= 512 * MIB).then(|| Self::resources_for_level(budget / GIB))
    }

    pub fn is_disk_space_low(system: &dyn System, env: &Environment) -> bool {
        system
            .get_available_space(Path::new(&env.get_instance_dir()))
            .is_some_and(|free| free < LOW_DISK_SPACE)
    }

    /// Raises any resource below the smallest usable size, returning a warning
    /// for each value that was changed.
    pub fn enforce_minimums(
        cpus: &mut u16,
        mem: &mut DataSize,
        disk: &mut DataSize,
    ) -> Vec<String> {
        let mut warnings = Vec::new();

        if *cpus < MIN_CPUS {
            warnings.push(format!(
                "CPUs raised from {cpus} to {MIN_CPUS} (minimum usable value)."
            ));
            *cpus = MIN_CPUS;
        }
        if mem.get_bytes() < MIN_MEMORY {
            warnings.push(format!(
                "Memory raised from {} to {} (minimum usable value).",
                mem.to_size(),
                DataSize::new(MIN_MEMORY).to_size()
            ));
            *mem = DataSize::new(MIN_MEMORY);
        }
        if disk.get_bytes() < MIN_DISK {
            warnings.push(format!(
                "Disk raised from {} to {} (minimum usable value).",
                disk.to_size(),
                DataSize::new(MIN_DISK).to_size()
            ));
            *disk = DataSize::new(MIN_DISK);
        }

        warnings
    }

    /// Map a level to its machine size. Level N is 2N vCPUs and N GiB, and the
    /// level 0 floor is 1 vCPU and 512 MiB.
    fn resources_for_level(level: usize) -> (u16, DataSize) {
        if level == 0 {
            (1, DataSize::new(512 * MIB))
        } else {
            ((level * 2) as u16, DataSize::new(level * GIB))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::SystemMock;

    #[test]
    fn test_read_from_host_uses_the_host_totals() {
        // A 16 GiB, 16 thread host. Only the totals are read, the available
        // memory plays no part in the default size.
        let system = SystemMock::new().set_host_resources((16 * GIB) as u64, (16 * GIB) as u64, 16);

        let (cpus, mem) = ResourceAllocator::read_from_host(&system).get_default_resources();

        assert_eq!(cpus, 6);
        assert_eq!(mem.get_bytes(), 3 * GIB);
    }

    #[test]
    fn test_get_default_resources() {
        for (host_gib, threads, cpus, mem) in [
            (8, 8, 2, GIB),
            (12, 12, 4, 2 * GIB),
            (16, 16, 6, 3 * GIB),
            (20, 20, 8, 4 * GIB),
            (23, 23, 8, 4 * GIB),
            (24, 24, 10, 5 * GIB),
            (28, 28, 12, 6 * GIB),
            (32, 32, 14, 7 * GIB),
            (36, 36, 16, 8 * GIB),
            (35, 35, 14, 7 * GIB),
            (40, 40, 18, 9 * GIB),
            (80, 80, 38, 19 * GIB),
            (8, 64, 2, GIB),
            (64, 8, 2, GIB),
            (4, 4, 1, 512 * MIB),
            (1, 1, 1, 512 * MIB),
        ] {
            let allocator = ResourceAllocator::new(host_gib * GIB, threads);
            let expected = (cpus, DataSize::new(mem));
            assert_eq!(
                allocator.get_default_resources(),
                expected,
                "{host_gib} GiB"
            );
        }
    }

    #[test]
    fn test_get_resources_for_budget() {
        for (available, expected) in [
            // 5 GiB available minus the 1 GiB reserve leaves a 4 GiB budget.
            (5 * GIB, Some((8, DataSize::new(4 * GIB)))),
            // 3.5 GiB available, 2.5 GiB budget, rounded down to level 2.
            (3 * GIB + 512 * MIB, Some((4, DataSize::new(2 * GIB)))),
            // A budget between 512 MiB and 1 GiB yields the smallest machine.
            (GIB + 512 * MIB, Some((1, DataSize::new(512 * MIB)))),
            (GIB, None),
            (0, None),
        ] {
            let resources = ResourceAllocator::get_resources_for_budget(available);
            assert_eq!(resources, expected, "available {available}");
        }
    }

    #[test]
    fn test_enforce_minimums_raises_values_below_the_floor() {
        let mut cpus = 0;
        let mut mem = DataSize::new(0);
        let mut disk = DataSize::new(0);

        let warnings = ResourceAllocator::enforce_minimums(&mut cpus, &mut mem, &mut disk);

        assert_eq!(cpus, MIN_CPUS);
        assert_eq!(mem.get_bytes(), MIN_MEMORY);
        assert_eq!(disk.get_bytes(), MIN_DISK);
        assert_eq!(warnings.len(), 3);
    }

    #[test]
    fn test_enforce_minimums_leaves_valid_values_untouched() {
        let mut cpus = 4;
        let mut mem = DataSize::new(4 * GIB);
        let mut disk = DataSize::new(50 * GIB);

        let warnings = ResourceAllocator::enforce_minimums(&mut cpus, &mut mem, &mut disk);

        assert_eq!(cpus, 4);
        assert_eq!(mem.get_bytes(), 4 * GIB);
        assert_eq!(disk.get_bytes(), 50 * GIB);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_enforce_minimums_raises_only_the_values_that_are_too_low() {
        let mut cpus = 0;
        let mut mem = DataSize::new(4 * GIB);
        let mut disk = DataSize::new(50 * GIB);

        let warnings = ResourceAllocator::enforce_minimums(&mut cpus, &mut mem, &mut disk);

        assert_eq!(cpus, MIN_CPUS);
        assert_eq!(mem.get_bytes(), 4 * GIB);
        assert_eq!(disk.get_bytes(), 50 * GIB);
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn test_enforce_minimums_accepts_a_value_exactly_at_the_floor() {
        let mut cpus = MIN_CPUS;
        let mut mem = DataSize::new(MIN_MEMORY);
        let mut disk = DataSize::new(MIN_DISK);

        let warnings = ResourceAllocator::enforce_minimums(&mut cpus, &mut mem, &mut disk);

        assert_eq!(cpus, MIN_CPUS);
        assert_eq!(mem.get_bytes(), MIN_MEMORY);
        assert_eq!(disk.get_bytes(), MIN_DISK);
        assert!(warnings.is_empty());
    }
}
