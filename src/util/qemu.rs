use crate::env::Environment;
use crate::error::Error;
use crate::fs::FS;
use crate::instance::Instance;
use crate::ssh_cmd::SshKeyGenerator;
use crate::util::SystemCommand;
use std::path::Path;

pub fn setup_cloud_init(env: &Environment, instance: &Instance) -> Result<(), Error> {
    let fs = FS::new();
    let name = &instance.name;
    let user = &instance.user;

    let user_data_img_path = env.get_user_data_image_file(&instance.name);

    if !Path::new(&user_data_img_path).exists() {
        let meta_data_path = env.get_meta_data_file(&instance.name);
        let user_data_path = env.get_user_data_file(&instance.name);

        fs.create_dir(&env.get_instance_cache_dir(&instance.name))?;

        if !Path::new(&meta_data_path).exists() {
            fs.write_file(
                &meta_data_path,
                format!("instance-id: {name}\nlocal-hostname: {name}\n").as_bytes(),
            )?;
        }

        if !Path::new(&user_data_path).exists() {
            let privatekey =
                Path::new(&env.get_instance_dir2(&instance.name)).join("ssh_client_key");
            let pubkey = privatekey
                .exists()
                .then(|| SshKeyGenerator::new().generate_public_key(&privatekey))
                .and_then(|key| key.ok())
                .unwrap_or_default();

            fs.write_file(
                &user_data_path,
                format!(
                    "\
                    #cloud-config\n\
                    users:\n\
                    \u{20}\u{20}- name: {user}\n\
                    \u{20}\u{20}\u{20}\u{20}lock_passwd: false\n\
                    \u{20}\u{20}\u{20}\u{20}hashed_passwd: $y$j9T$wifmOLBedd7NSaH2IqG4L.$2J.8E.qE57lxapsWosOFod37djHePHg7Go03iDNsRe4\n\
                    \u{20}\u{20}\u{20}\u{20}ssh-authorized-keys: [{pubkey}]\n\
                    \u{20}\u{20}\u{20}\u{20}shell: /bin/bash\n\
                    \u{20}\u{20}\u{20}\u{20}sudo: ALL=(ALL) NOPASSWD:ALL\n\
                    package_update: true\n\
                    packages:\n\
                    \u{20}\u{20}- openssh\n\
                    \u{20}\u{20}- qemu-guest-agent\n\
                    runcmd:\n\
                    \u{20}\u{20}- \
                        systemctl enable --now qemu-guest-agent\n\
                "
                )
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
