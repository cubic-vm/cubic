use crate::cloudinit::{MetaDataFactory, UserDataFactory};
use crate::error::Result;
use crate::iso9660::IsoWriter;
use crate::models::{Environment, Instance};
use crate::platform::System;
use crate::ssh::SshKeyGenerator;
use std::io::Cursor;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct UserDataImageFactory;

impl UserDataImageFactory {
    pub fn create_rust(
        &self,
        system: &dyn System,
        env: &Environment,
        instance: &Instance,
    ) -> Result<()> {
        let user_data_img_path = PathBuf::from(env.get_user_data_image_file(&instance.name));

        if system.exists_path(&user_data_img_path) {
            return Ok(());
        }

        // Generate SSH public key
        let privatekey = Path::new(&env.get_instance_dir2(&instance.name)).join("ssh_client_key");
        let pubkey = system
            .exists_path(&privatekey)
            .then(|| SshKeyGenerator::new().generate_public_key(system, &privatekey))
            .and_then(|key| key.ok())
            .unwrap_or_default();

        // Generate Cloud Init files
        let meta_data = MetaDataFactory.create(&instance.name);
        let user_data =
            UserDataFactory.create(&instance.user, &pubkey, instance.execute.as_deref());

        // Generate ISO file
        system.create_dir(Path::new(&env.get_instance_cache_dir(&instance.name)))?;
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

        let mut buffer = Cursor::new(Vec::new());
        iso_writer.create_iso(&mut buffer)?;
        system.write_file(&user_data_img_path, buffer.get_ref())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::UserName;
    use crate::platform::SystemMock;
    use std::str::FromStr;

    fn build_env() -> Environment {
        Environment::new(
            UserName::from_str("cubic").unwrap(),
            "/data".to_string(),
            "/cache".to_string(),
            "/run".to_string(),
        )
    }

    fn build_instance() -> Instance {
        Instance {
            name: "test".to_string(),
            ..Instance::default()
        }
    }

    #[test]
    fn test_create_rust_writes_the_cloud_init_image() {
        let system = SystemMock::new();
        let env = build_env();

        UserDataImageFactory
            .create_rust(&system, &env, &build_instance())
            .unwrap();

        let image = system
            .get_written_file(&env.get_user_data_image_file("test"))
            .expect("expected the cloud init image to have been written");
        assert!(!image.is_empty());
    }

    #[test]
    fn test_create_rust_embeds_the_public_key_of_the_instance() {
        let system = SystemMock::new();
        let env = build_env();
        let key_path = Path::new(&env.get_instance_dir2("test")).join("ssh_client_key");
        SshKeyGenerator::new()
            .generate_key(&system, &key_path)
            .unwrap();

        UserDataImageFactory
            .create_rust(&system, &env, &build_instance())
            .unwrap();

        let pubkey = SshKeyGenerator::new()
            .generate_public_key(&system, &key_path)
            .unwrap();
        let image = system
            .get_written_file(&env.get_user_data_image_file("test"))
            .unwrap();
        assert!(
            image
                .windows(pubkey.len())
                .any(|window| window == pubkey.as_bytes())
        );
    }

    #[test]
    fn test_create_rust_keeps_an_existing_image() {
        let env = build_env();
        let system = SystemMock::new().add_file(&env.get_user_data_image_file("test"), b"existing");

        UserDataImageFactory
            .create_rust(&system, &env, &build_instance())
            .unwrap();

        assert_eq!(
            system.get_written_file(&env.get_user_data_image_file("test")),
            Some(b"existing".to_vec())
        );
    }
}
