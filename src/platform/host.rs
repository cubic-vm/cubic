pub trait Host {
    fn read_env_var(&self, key: &str) -> Option<String>;

    fn get_total_memory(&self) -> u64;
    fn get_available_memory(&self) -> u64;
    fn get_cpu_count(&self) -> u16;
}
