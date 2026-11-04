use crate::actions::StoreInstanceAction;
use crate::commands::Context;
use crate::error::Result;
use crate::models::{DataSize, Instance};
use crate::qemu::QemuImg;
use crate::ssh::SshKeyGenerator;
use std::path::Path;

#[derive(Default)]
pub struct CreateInstanceAction;

impl CreateInstanceAction {
    pub fn new() -> Self {
        Self
    }

    pub fn run(
        &mut self,
        context: &Context,
        image_path: &Path,
        instance: Instance,
        disk: Option<&DataSize>,
        auto_remove: bool,
    ) -> Result<()> {
        let system = context.get_system();
        let env = context.get_env();
        let tmp_name = format!("{}.tmp", instance.name);
        let target_dir = &env.get_instance_dir2(&instance.name);
        let tmp_dir = &env.get_instance_dir2(&tmp_name);

        let result = self
            .build_staging_dir(context, &tmp_name, image_path, instance, disk, auto_remove)
            .and_then(|()| system.rename_file(tmp_dir, target_dir));

        // Free the space of a partial image, the error that broke the create wins
        if result.is_err() {
            system.remove_dir(tmp_dir).ok();
        }
        result
    }

    fn build_staging_dir(
        &self,
        context: &Context,
        tmp_name: &str,
        image_path: &Path,
        mut instance: Instance,
        disk: Option<&DataSize>,
        auto_remove: bool,
    ) -> Result<()> {
        let system = context.get_system();
        let env = context.get_env();
        let tmp_image = &env.get_instance_image_file(tmp_name);
        let tmp_config = &env.get_instance_toml_config_file(tmp_name);

        // Create directory
        system.create_dir(&env.get_instance_dir2(tmp_name))?;

        // Create SSH key
        SshKeyGenerator::new().generate_key(system, &env.get_ssh_private_key_file(tmp_name))?;

        // Create virtual machine instance image file
        if auto_remove {
            QemuImg::new(system).create_overlay(image_path, tmp_image)?;
        } else {
            system.copy_file(image_path, tmp_image)?;
        }

        if let Some(disk) = disk {
            QemuImg::new(system).resize(tmp_image, disk.get_bytes() as u64)?;
        }

        // Write configuration file
        instance.auto_remove = auto_remove;
        instance.first_boot = true;
        StoreInstanceAction::new().write_to(context, &instance, tmp_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::ContextBuilder;
    use crate::models::Environment;
    use crate::platform::{FileSystem, SystemMock};
    use std::sync::Arc;

    #[test]
    fn test_run_removes_the_staging_dir_when_a_step_fails() {
        let builder = ContextBuilder::new(SystemMock::new());
        let system = Arc::clone(builder.get_system());
        let context = builder.build();
        let instance = Instance {
            name: "test".to_string(),
            ..Instance::default()
        };

        // The image is missing, so copying it fails after the dir and key exist
        let image = Path::new("/missing.img");
        let result = CreateInstanceAction::new().run(&context, image, instance, None, false);

        assert!(result.is_err());
        let tmp_dir = Environment::default().get_instance_dir2("test.tmp");
        assert!(!system.exists_path(&tmp_dir));
    }
}
