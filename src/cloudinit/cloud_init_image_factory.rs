use crate::cloudinit::iso_image::IsoImage;
use crate::cloudinit::{MetaDataFactory, UserDataFactory};
use crate::error::Result;
use crate::models::{Environment, Instance};
use crate::platform::{Access, System};
use crate::ssh::SshKeyGenerator;

#[derive(Default)]
pub struct CloudInitImageFactory;

impl CloudInitImageFactory {
    pub fn create(
        &self,
        system: &dyn System,
        env: &Environment,
        instance: &Instance,
    ) -> Result<()> {
        let cloud_init_path = env.get_cloud_init_file(&instance.name);

        if system.exists_path(&cloud_init_path) {
            return Ok(());
        }

        // Generate SSH public key
        let privatekey = env.get_ssh_private_key_file(&instance.name);
        let pubkey = system
            .exists_path(&privatekey)
            .then(|| SshKeyGenerator::new().generate_public_key(system, &privatekey))
            .and_then(|key| key.ok())
            .unwrap_or_default();

        // Generate Cloud Init files
        let meta_data = MetaDataFactory.create(&instance.name);
        let timezone = system.get_timezone_name().unwrap_or_default();
        let user_data = UserDataFactory.create(instance, &pubkey, &timezone);

        // Generate ISO file
        let files = [
            ("meta-data", meta_data.as_bytes()),
            ("user-data", user_data.as_bytes()),
        ];
        let image = IsoImage::build("cidata", &files);
        system.write_file(&cloud_init_path, &image, Access::Normal)?;
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
            "/data".into(),
            "/cache".into(),
        )
    }

    fn build_instance() -> Instance {
        Instance {
            name: "test".to_string(),
            ..Instance::default()
        }
    }

    #[test]
    fn test_create_embeds_the_public_key_of_the_instance() {
        let system = SystemMock::new();
        let env = build_env();
        let key_path = env.get_ssh_private_key_file("test");
        SshKeyGenerator::new()
            .generate_key(&system, &key_path)
            .unwrap();

        CloudInitImageFactory
            .create(&system, &env, &build_instance())
            .unwrap();

        let pubkey = SshKeyGenerator::new()
            .generate_public_key(&system, &key_path)
            .unwrap();
        let image = system
            .get_written_file(env.get_cloud_init_file("test"))
            .unwrap();
        assert!(
            image
                .windows(pubkey.len())
                .any(|window| window == pubkey.as_bytes())
        );
    }

    #[test]
    fn test_create_sets_the_host_timezone_or_utc() {
        let env = build_env();

        let system = SystemMock::new().set_host_timezone("Region/City");
        CloudInitImageFactory
            .create(&system, &env, &build_instance())
            .unwrap();
        let image = system
            .get_written_file(env.get_cloud_init_file("test"))
            .unwrap();
        assert!(String::from_utf8_lossy(&image).contains("timezone: Region/City\n"));

        let system = SystemMock::new();
        CloudInitImageFactory
            .create(&system, &env, &build_instance())
            .unwrap();
        let image = system
            .get_written_file(env.get_cloud_init_file("test"))
            .unwrap();
        assert!(String::from_utf8_lossy(&image).contains("timezone: UTC\n"));
    }

    #[test]
    fn test_create_keeps_an_existing_image() {
        let env = build_env();
        let system = SystemMock::new().add_file(env.get_cloud_init_file("test"), b"existing");

        CloudInitImageFactory
            .create(&system, &env, &build_instance())
            .unwrap();

        let written = system.get_written_file(env.get_cloud_init_file("test"));
        assert_eq!(written, Some(b"existing".to_vec()));
    }
}
