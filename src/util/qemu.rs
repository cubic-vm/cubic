use crate::env::Environment;
use crate::error::{Error, Result};
use crate::fs::FS;
use crate::instance::Instance;
use crate::ssh_cmd::SshKeyGenerator;
use crate::util::SystemCommand;
use std::io::Write;
use std::path::Path;

fn write_meta_data(out: &mut dyn Write, name: &str) -> Result<()> {
    out.write_all(format!("instance-id: {name}\nlocal-hostname: {name}\n").as_bytes())
        .map_err(Error::from)
}

fn write_user_data(out: &mut dyn Write, user: &str, pubkey: &str) -> Result<()> {
    out.write_all(
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
            bootcmd:\n\
            \u{20}\u{20}- systemctl enable --now qemu-guest-agent\n\
            runcmd:\n\
            \u{20}\u{20}- \
                systemctl enable --now qemu-guest-agent\n\
        "
        )
        .as_bytes(),
    ).map_err(Error::from)
}

pub fn setup_cloud_init(env: &Environment, instance: &Instance) -> Result<()> {
    let fs = FS::new();
    let name = &instance.name;
    let user = &instance.user;

    let user_data_img_path = env.get_user_data_image_file(&instance.name);

    if !Path::new(&user_data_img_path).exists() {
        let meta_data_path = env.get_meta_data_file(&instance.name);
        let user_data_path = env.get_user_data_file(&instance.name);

        fs.create_dir(&env.get_instance_cache_dir(&instance.name))?;

        if !Path::new(&meta_data_path).exists() {
            write_meta_data(&mut fs.create_file(&meta_data_path)?, name)?;
        }

        if !Path::new(&user_data_path).exists() {
            let privatekey =
                Path::new(&env.get_instance_dir2(&instance.name)).join("ssh_client_key");
            let pubkey = privatekey
                .exists()
                .then(|| SshKeyGenerator::new().generate_public_key(&privatekey))
                .and_then(|key| key.ok())
                .unwrap_or_default();

            write_user_data(&mut fs.create_file(&user_data_path)?, user, &pubkey)?;
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

#[cfg(test)]
mod tests {
    pub use super::*;
    use std::io::BufWriter;

    #[test]
    fn test_meta_user_data() {
        let mut buffer = BufWriter::new(Vec::new());
        write_meta_data(&mut buffer, "myinstance").unwrap();
        let actual = String::from_utf8(buffer.into_inner().unwrap()).unwrap();

        assert_eq!(
            actual,
            "instance-id: myinstance\nlocal-hostname: myinstance\n"
        );
    }

    #[test]
    fn test_write_user_data() {
        let mut buffer = BufWriter::new(Vec::new());
        write_user_data(&mut buffer, "tux", "pubkey").unwrap();
        let actual = String::from_utf8(buffer.into_inner().unwrap()).unwrap();

        let expected = r#"#cloud-config
users:
  - name: tux
    lock_passwd: false
    hashed_passwd: $y$j9T$wifmOLBedd7NSaH2IqG4L.$2J.8E.qE57lxapsWosOFod37djHePHg7Go03iDNsRe4
    ssh-authorized-keys: [pubkey]
    shell: /bin/bash
    sudo: ALL=(ALL) NOPASSWD:ALL
package_update: true
packages:
  - openssh
  - qemu-guest-agent
bootcmd:
  - systemctl enable --now qemu-guest-agent
runcmd:
  - systemctl enable --now qemu-guest-agent
"#;
        assert_eq!(
            actual, expected,
            "\nActual: {actual}\nExpected: {expected}\n"
        )
    }
}
