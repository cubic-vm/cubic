use crate::cloudinit::{MetaDataFactory, UserDataFactory};
use crate::env::Environment;
use crate::error::Result;
use crate::fs::FS;
use crate::instance::Instance;
use crate::iso9660::IsoWriter;
use crate::ssh_cmd::SshKeyGenerator;
use std::path::Path;

#[derive(Default)]
pub struct UserDataImageFactory;

impl UserDataImageFactory {
    pub fn create_rust(&self, env: &Environment, instance: &Instance) -> Result<()> {
        let user_data_img_path = env.get_user_data_image_file(&instance.name);
        let fs = FS::new();

        if Path::new(&user_data_img_path).exists() {
            return Ok(());
        }

        // Generate SSH public key
        let privatekey = Path::new(&env.get_instance_dir2(&instance.name)).join("ssh_client_key");
        let pubkey = privatekey
            .exists()
            .then(|| SshKeyGenerator::new().generate_public_key(&privatekey))
            .and_then(|key| key.ok())
            .unwrap_or_default();

        // Generate Cloud Init files
        let meta_data = MetaDataFactory.create(&instance.name);
        let user_data =
            UserDataFactory.create(&instance.user, &pubkey, instance.execute.as_deref());

        // Generate ISO file
        fs.create_dir(&env.get_instance_cache_dir(&instance.name))?;
        let mut iso_writer = IsoWriter::new();
        iso_writer.pvd.system_id = "LINUX".to_string();
        iso_writer.pvd.volume_id = "cidata".to_string();
        iso_writer.pvd.application_id = "Cubic".to_string();
        iso_writer
            .files
            .insert("meta-data".to_string(), meta_data.into_bytes());
        iso_writer
            .files
            .insert("user-data".to_string(), user_data.into_bytes());
        iso_writer.create_iso(&user_data_img_path)?;
        Ok(())
    }
}
