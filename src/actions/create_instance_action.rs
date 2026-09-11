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
        image_path: &str,
        mut instance: Instance,
        auto_remove: bool,
    ) -> Result<()> {
        let system = context.get_system();
        let instance_name = instance.name.clone();
        let target_dir = &context.get_env().get_instance_dir2(&instance.name);
        let tmp_dir = &format!("{target_dir}.tmp");
        let tmp_image = &format!("{tmp_dir}/machine.img");

        // Create directory
        system.create_dir(Path::new(tmp_dir))?;

        // Create SSH key
        SshKeyGenerator::new().generate_key(system, &Path::new(tmp_dir).join("ssh_client_key"))?;

        // Create virtual machine instance image file
        if auto_remove {
            QemuImg::new(system).create_overlay(image_path, tmp_image)?;
        } else {
            system.copy_file(Path::new(image_path), Path::new(tmp_image))?;
        }

        // Set disk capacity
        QemuImg::new(system).resize(tmp_image, instance.disk_capacity.get_bytes() as u64)?;

        // Write configuration file
        instance.auto_remove = auto_remove;
        instance.name = format!("{instance_name}.tmp");
        context.get_instance_store().store(&instance)?;
        instance.name = instance_name;

        system.rename_file(Path::new(tmp_dir), Path::new(target_dir))
    }
}
