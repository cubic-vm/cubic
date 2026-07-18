use crate::error::Result;
use crate::instance::InstanceStore;
use crate::models::{TargetInstancePath, TargetPath};

pub fn resolve_target_path(
    tp: &TargetPath,
    instance_store: &dyn InstanceStore,
) -> Result<TargetInstancePath> {
    if let Some(target) = tp.get_target() {
        let instance = instance_store.load(target.get_instance().as_str())?;
        Ok(TargetInstancePath {
            user: target.get_user().map(|user| user.to_string()),
            instance: Some(instance),
            path: tp.path.clone(),
        })
    } else {
        Ok(TargetInstancePath {
            user: None,
            instance: None,
            path: tp.path.clone(),
        })
    }
}
