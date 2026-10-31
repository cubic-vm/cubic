use crate::commands::Context;
use crate::error::Result;
use crate::models::Instance;
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
        mut instance: Instance,
        auto_remove: bool,
    ) -> Result<()> {
        let system = context.get_system();
        let env = context.get_env();
        let instance_name = instance.name.clone();
        let tmp_name = format!("{instance_name}.tmp");
        let target_dir = &env.get_instance_dir2(&instance_name);
        let tmp_dir = &env.get_instance_dir2(&tmp_name);
        let tmp_image = &env.get_instance_image_file(&tmp_name);

        // Create directory
        system.create_dir(tmp_dir)?;

        // Create SSH key
        SshKeyGenerator::new().generate_key(system, &env.get_ssh_private_key_file(&tmp_name))?;

        // Create virtual machine instance image file
        if auto_remove {
            QemuImg::new(system).create_overlay(image_path, tmp_image)?;
        } else {
            system.copy_file(image_path, tmp_image)?;
        }

        // Set disk capacity
        QemuImg::new(system).resize(tmp_image, instance.disk_capacity.get_bytes() as u64)?;

        // Write configuration file
        instance.auto_remove = auto_remove;
        instance.first_boot = true;
        instance.name = tmp_name;
        context.get_instance_store().store(&instance)?;
        instance.name = instance_name;

        system.rename_file(tmp_dir, target_dir)
    }
}
