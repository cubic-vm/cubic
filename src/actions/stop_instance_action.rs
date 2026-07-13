use crate::error::{Error, Result};
use crate::instance::InstanceStore;
use crate::models::Instance;

pub struct StopInstanceAction {
    instance: Instance,
}

impl StopInstanceAction {
    pub fn new(instance: &Instance) -> Self {
        Self {
            instance: instance.clone(),
        }
    }

    pub fn run(&mut self, instance_dao: &dyn InstanceStore, kill: bool) -> Result<()> {
        if !instance_dao.exists(&self.instance.name) {
            return Err(Error::UnknownInstance(self.instance.name.clone()));
        }

        if instance_dao.is_running(&self.instance) {
            if kill {
                instance_dao.kill(&self.instance)?;
            } else {
                instance_dao.get_monitor(&self.instance)?.shutdown()?;
            }
        }

        Ok(())
    }

    pub fn is_done(&self, instance_dao: &dyn InstanceStore) -> bool {
        !instance_dao.is_running(&self.instance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::InstanceStoreMock;

    fn build_instance(name: &str) -> Instance {
        Instance {
            name: name.to_string(),
            ..Instance::default()
        }
    }

    #[test]
    fn test_stop_rejects_unknown_instance() {
        let store = InstanceStoreMock::new(Vec::new());

        let result = StopInstanceAction::new(&build_instance("missing")).run(&store, false);

        assert!(matches!(
            result,
            Err(Error::UnknownInstance(ref name)) if name == "missing"
        ));
    }

    #[test]
    fn test_stop_kills_running_instance() {
        let instance = build_instance("test");
        let store = InstanceStoreMock::new_with_running(vec![instance.clone()], &["test"]);

        StopInstanceAction::new(&instance)
            .run(&store, true)
            .unwrap();

        assert_eq!(store.get_killed(), ["test"]);
    }

    #[test]
    fn test_stop_skips_stopped_instance() {
        let instance = build_instance("test");
        let store = InstanceStoreMock::new(vec![instance.clone()]);

        StopInstanceAction::new(&instance)
            .run(&store, true)
            .unwrap();

        assert!(store.get_killed().is_empty());
    }

    #[test]
    fn test_stop_without_kill_requests_monitor_shutdown() {
        let instance = build_instance("test");
        let store = InstanceStoreMock::new_with_running(vec![instance.clone()], &["test"]);

        // The mock has no monitor, so reaching the monitor path surfaces
        // its InstanceNotRunning error instead of a kill.
        let result = StopInstanceAction::new(&instance).run(&store, false);

        assert!(result.is_err());
        assert!(store.get_killed().is_empty());
    }

    #[test]
    fn test_is_done_when_instance_is_stopped() {
        let instance = build_instance("test");
        let store = InstanceStoreMock::new(vec![instance.clone()]);

        assert!(StopInstanceAction::new(&instance).is_done(&store));
    }

    #[test]
    fn test_is_not_done_while_instance_is_running() {
        let instance = build_instance("test");
        let store = InstanceStoreMock::new_with_running(vec![instance.clone()], &["test"]);

        assert!(!StopInstanceAction::new(&instance).is_done(&store));
    }
}
