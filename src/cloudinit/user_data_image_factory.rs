use crate::cloudinit::{MetaDataFactory, UserDataFactory};
use crate::env::Environment;
use crate::error::Result;
use crate::fs::FS;
use crate::instance::Instance;
use crate::ssh_cmd::SshKeyGenerator;
use crate::util::SystemCommand;
use std::io::Write;
use std::path::Path;

#[derive(Default)]
pub struct UserDataImageFactory;

impl UserDataImageFactory {
    pub fn create(&self, env: &Environment, instance: &Instance) -> Result<()> {
        let fs = FS::new();
        let name = &instance.name;
        let user = &instance.user;

        let user_data_img_path = env.get_user_data_image_file(&instance.name);

        if !Path::new(&user_data_img_path).exists() {
            let meta_data_path = env.get_meta_data_file(&instance.name);
            let user_data_path = env.get_user_data_file(&instance.name);

            fs.create_dir(&env.get_instance_cache_dir(&instance.name))?;

            if !Path::new(&meta_data_path).exists() {
                fs.create_file(&meta_data_path)?
                    .write_all(MetaDataFactory.create(name).as_bytes())?;
            }

            if !Path::new(&user_data_path).exists() {
                let privatekey =
                    Path::new(&env.get_instance_dir2(&instance.name)).join("ssh_client_key");
                let pubkey = privatekey
                    .exists()
                    .then(|| SshKeyGenerator::new().generate_public_key(&privatekey))
                    .and_then(|key| key.ok())
                    .unwrap_or_default();

                fs.create_file(&user_data_path)?.write_all(
                    UserDataFactory
                        .create(user, &pubkey, instance.execute.as_deref())
                        .as_bytes(),
                )?;
            }

            SystemCommand::new("mkisofs")
                .arg("-RJ")
                .arg("-V")
                .arg("cidata")
                .arg("-o")
                .arg(&user_data_img_path)
                .arg("-graft-points")
                .arg(format!("/={user_data_path}"))
                .arg(format!("/={meta_data_path}"))
                .run()?;
        }

        Result::Ok(())
    }
}
