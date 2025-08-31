use crate::env::Environment;
use crate::error::Error;
use crate::fs::FS;
use crate::image::Image;
use crate::instance::{Instance, InstanceStore};
use crate::util::SystemCommand;

#[derive(Default)]
pub struct CreateInstanceAction;

impl CreateInstanceAction {
    pub fn new() -> Self {
        Self
    }

    pub fn run(
        &mut self,
        env: &Environment,
        fs: &FS,
        instance_store: &dyn InstanceStore,
        image: &Image,
        mut instance: Instance,
    ) -> Result<(), Error> {
        let instance_name = instance.name.clone();
        let target_dir = &env.get_instance_dir2(&instance.name);
        let tmp_dir = &format!("{target_dir}.tmp");
        let tmp_image = &format!("{tmp_dir}/machine.img");

        // Create virtual machine instance image file
        fs.create_dir(tmp_dir)?;
        SystemCommand::new("qemu-img")
            .arg("convert")
            .arg("-f")
            .arg("qcow2")
            .arg("-O")
            .arg("qcow2")
            .arg(env.get_image_file(&image.to_file_name()))
            .arg(tmp_image)
            .run()?;

        // Set disk capacity
        SystemCommand::new("qemu-img")
            .arg("resize")
            .arg(tmp_image)
            .arg(instance.disk_capacity.to_string())
            .run()?;

        // Write configuration file
        instance.name = format!("{instance_name}.tmp");
        instance_store.store(&instance)?;
        instance.name = instance_name;

        fs.rename_file(tmp_dir, target_dir)
    }
}
