#[cfg(test)]
pub mod tests {
    use crate::platform::System;
    use std::collections::HashMap;

    pub struct SystemMock {
        env_vars: HashMap<String, String>,
    }

    impl SystemMock {
        pub fn new() -> Self {
            Self {
                env_vars: HashMap::new(),
            }
        }

        pub fn add_env_var(mut self, key: &str, value: &str) -> Self {
            self.env_vars.insert(key.to_string(), value.to_string());
            self
        }
    }

    impl System for SystemMock {
        fn read_env_var(&self, key: &str) -> Option<String> {
            self.env_vars.get(key).cloned()
        }
    }

    #[test]
    fn read_env_var_returns_configured_value() {
        let system = SystemMock::new().add_env_var("FOO", "bar");

        assert_eq!(system.read_env_var("FOO"), Some("bar".to_string()));
    }

    #[test]
    fn read_env_var_returns_none_when_not_set() {
        let system = SystemMock::new();

        assert_eq!(system.read_env_var("FOO"), None);
    }
}
