use crate::commands::Context;
use crate::error::Result;
use crate::models::Instance;
use crate::qemu::QemuImg;
use crate::ssh_cmd::SshKeyGenerator;
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

        let qemu_img = QemuImg::new(system);

        // Create virtual machine instance image file
        qemu_img.convert(image_path, tmp_image)?;

        // Set disk capacity
        qemu_img.resize(tmp_image, instance.disk_capacity.get_bytes() as u64)?;

        // Write configuration file
        instance.name = format!("{instance_name}.tmp");
        context.get_instance_store().store(&instance)?;
        instance.name = instance_name;

        system.rename_file(Path::new(tmp_dir), Path::new(target_dir))
    }
}
