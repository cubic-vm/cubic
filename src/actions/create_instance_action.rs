use crate::commands::Context;
use crate::error::Result;
use crate::fs::FS;
use crate::instance::Instance;
use crate::ssh_cmd::SshKeyGenerator;
use crate::util::SystemCommand;
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
        fs: &FS,
        image_path: &str,
        mut instance: Instance,
    ) -> Result<()> {
        let instance_name = instance.name.clone();
        let target_dir = &context.get_env().get_instance_dir2(&instance.name);
        let tmp_dir = &format!("{target_dir}.tmp");
        let tmp_image = &format!("{tmp_dir}/machine.img");

        // Create directory
        fs.create_dir(tmp_dir)?;

        // Create SSH key
        SshKeyGenerator::new().generate_key(&Path::new(tmp_dir).join("ssh_client_key"))?;

        // Create virtual machine instance image file
        SystemCommand::new("qemu-img")
            .arg("convert")
            .arg("-f")
            .arg("qcow2")
            .arg("-O")
            .arg("qcow2")
            .arg(image_path)
            .arg(tmp_image)
            .run()?;

        // Set disk capacity
        SystemCommand::new("qemu-img")
            .arg("resize")
            .arg(tmp_image)
            .arg(instance.disk_capacity.get_bytes().to_string())
            .run()?;

        // Write configuration file
        instance.name = format!("{instance_name}.tmp");
        context.get_instance_store().store(&instance)?;
        instance.name = instance_name;

        fs.rename_file(tmp_dir, target_dir)
    }
}
