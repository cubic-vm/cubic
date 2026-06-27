use crate::models::DataSize;
use sysinfo::System;

const MIB: usize = 1024 * 1024;
const GIB: usize = 1024 * MIB;

/// Memory kept aside for the host and QEMU overhead so a machine never claims
/// all of the available memory.
pub const HOST_MEMORY_RESERVE: usize = GIB;

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
    /// `cpus().len()` counts logical processors, so a host with simultaneous
    /// multithreading reports its thread count rather than its physical cores.
    /// This matches the vCPU count handed to a machine, which also maps to
    /// threads.
    pub fn read_from_host() -> Self {
        let mut system = System::new();
        system.refresh_memory();
        system.refresh_cpu_all();
        Self::new(system.total_memory() as usize, system.cpus().len() as u16)
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

    fn assert_resources(
        host_mem_gib: usize,
        host_threads: u16,
        expected_cpus: u16,
        expected_mem_bytes: usize,
    ) {
        let allocator = ResourceAllocator::new(host_mem_gib * GIB, host_threads);
        let (cpus, mem) = allocator.get_default_resources();
        assert_eq!(cpus, expected_cpus);
        assert_eq!(mem.get_bytes(), expected_mem_bytes);
    }

    #[test]
    fn test_level_8() {
        assert_resources(36, 36, 16, 8 * GIB);
    }

    #[test]
    fn test_level_keeps_scaling_without_a_maximum() {
        assert_resources(40, 40, 18, 9 * GIB);
        assert_resources(80, 80, 38, 19 * GIB);
    }

    #[test]
    fn test_level_7() {
        assert_resources(32, 32, 14, 7 * GIB);
    }

    #[test]
    fn test_level_6() {
        assert_resources(28, 28, 12, 6 * GIB);
    }

    #[test]
    fn test_level_5() {
        assert_resources(24, 24, 10, 5 * GIB);
    }

    #[test]
    fn test_level_4() {
        assert_resources(20, 20, 8, 4 * GIB);
        assert_resources(23, 23, 8, 4 * GIB);
    }

    #[test]
    fn test_level_3() {
        assert_resources(16, 16, 6, 3 * GIB);
    }

    #[test]
    fn test_level_2() {
        assert_resources(12, 12, 4, 2 * GIB);
    }

    #[test]
    fn test_level_1() {
        assert_resources(8, 8, 2, GIB);
    }

    #[test]
    fn test_just_below_a_threshold_stays_on_the_lower_level() {
        assert_resources(35, 35, 14, 7 * GIB);
    }

    #[test]
    fn test_memory_threshold_dominates() {
        assert_resources(8, 64, 2, GIB);
    }

    #[test]
    fn test_thread_threshold_dominates() {
        assert_resources(64, 8, 2, GIB);
    }

    #[test]
    fn test_small_host_falls_back_to_one_core() {
        assert_resources(4, 4, 1, 512 * MIB);
        assert_resources(1, 1, 1, 512 * MIB);
    }

    #[test]
    fn test_budget_reserves_host_memory() {
        // 5 GiB available minus the 1 GiB reserve leaves a 4 GiB budget.
        assert_eq!(
            ResourceAllocator::get_resources_for_budget(5 * GIB),
            Some((8, DataSize::new(4 * GIB)))
        );
    }

    #[test]
    fn test_budget_rounds_down_to_whole_gib() {
        // 3.5 GiB available, 2.5 GiB budget, fits level 2.
        assert_eq!(
            ResourceAllocator::get_resources_for_budget(3 * GIB + 512 * MIB),
            Some((4, DataSize::new(2 * GIB)))
        );
    }

    #[test]
    fn test_budget_falls_back_to_512_mib() {
        // Budget between 512 MiB and 1 GiB yields the smallest machine.
        assert_eq!(
            ResourceAllocator::get_resources_for_budget(GIB + 512 * MIB),
            Some((1, DataSize::new(512 * MIB)))
        );
    }

    #[test]
    fn test_budget_too_small_returns_none() {
        assert_eq!(ResourceAllocator::get_resources_for_budget(GIB), None);
        assert_eq!(ResourceAllocator::get_resources_for_budget(0), None);
    }
}
