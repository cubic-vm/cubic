pub trait System {
    fn read_env_var(&self, key: &str) -> Option<String>;
}

#[derive(Default)]
pub struct OsSystem;

impl OsSystem {
    pub fn new() -> Self {
        Self
    }
}

impl System for OsSystem {
    fn read_env_var(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
}
