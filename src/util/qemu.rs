use crate::error::Error;
use crate::fs::FS;
use crate::instance::{Instance, MountPoint};
use crate::ssh_cmd::get_ssh_pub_keys;
use std::path::Path;
use std::process::{Command, Stdio};
use std::str;

pub fn bytes_to_human_readable(bytes: u64) -> String {
    match bytes.checked_ilog(1024) {
        Some(1) => format!("{:3.1} KiB", bytes as f64 / 1024_f64.powf(1_f64)),
        Some(2) => format!("{:3.1} MiB", bytes as f64 / 1024_f64.powf(2_f64)),
        Some(3) => format!("{:3.1} GiB", bytes as f64 / 1024_f64.powf(3_f64)),
        Some(4) => format!("{:3.1} TiB", bytes as f64 / 1024_f64.powf(4_f64)),
        _ => format!("{:3.1}   B", bytes as f64),
    }
}

pub fn human_readable_to_bytes(size: &str) -> Result<u64, Error> {
    if size.is_empty() {
        return Result::Err(Error::CannotParseSize(size.to_string()));
    }

    let suffix: char = size.bytes().last().unwrap() as char;
    let size = &size[..size.len() - 1];
    let power = match suffix {
        'B' => 0,
        'K' => 1,
        'M' => 2,
        'G' => 3,
        'T' => 4,
        _ => return Result::Err(Error::CannotParseSize(size.to_string())),
    };

    size.parse()
        .map(|size: u64| size * 1024_u64.pow(power))
        .map_err(|_| Error::CannotParseSize(size.to_string()))
}

pub fn setup_cloud_init(instance: &Instance, dir: &str, force: bool) -> Result<(), Error> {
    let fs = FS::new();
    let name = &instance.name;
    let user = &instance.user;

    let user_data_img_path = format!("{dir}/user-data.img");

    if force || !Path::new(&user_data_img_path).exists() {
        let meta_data_path = format!("{dir}/meta-data");
        let user_data_path = format!("{dir}/user-data");

        fs.create_dir(dir)?;

        if force || !Path::new(&meta_data_path).exists() {
            fs.write_file(
                &meta_data_path,
                format!("instance-id: {name}\nlocal-hostname: {name}\n").as_bytes(),
            )?;
        }

        let mut bootcmds = String::new();
        if !instance.mounts.is_empty() {
            bootcmds += "bootcmd:\n";
            for (index, MountPoint { guest, .. }) in instance.mounts.iter().enumerate() {
                bootcmds += &format!("  - mount -t 9p cubic{index} {guest}\n");
            }
        }

        if force || !Path::new(&user_data_path).exists() {
            let ssh_pk = if let Ok(ssh_keys) = get_ssh_pub_keys() {
                format!(
                    "\u{20}\u{20}\u{20}\u{20}ssh-authorized-keys:\n{}",
                    ssh_keys
                        .iter()
                        .map(|key| format!("\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}- {key}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            } else {
                String::new()
            };

            fs.write_file(
                &user_data_path,
                format!(
                    "\
                    #cloud-config\n\
                    users:\n\
                    \u{20}\u{20}- name: {user}\n\
                    \u{20}\u{20}\u{20}\u{20}lock_passwd: false\n\
                    \u{20}\u{20}\u{20}\u{20}hashed_passwd: $y$j9T$wifmOLBedd7NSaH2IqG4L.$2J.8E.qE57lxapsWosOFod37djHePHg7Go03iDNsRe4\n\
                    {ssh_pk}\n\
                    \u{20}\u{20}\u{20}\u{20}shell: /bin/bash\n\
                    \u{20}\u{20}\u{20}\u{20}sudo: ALL=(ALL) NOPASSWD:ALL\n\
                    ssh_pwauth: True\n\
                    packages:\n\
                    \u{20}\u{20}- openssh\n\
                    {bootcmds}\n\
                    runcmd:\n\
                    \u{20}\u{20}- \
                        apt update; apt install -y qemu-guest-agent socat; \
                        dnf install -y qemu-guest-agent socat; \
                        yes | pacman -S qemu-guest-agent socat; \
                        systemctl enable --now qemu-guest-agent\n\
                "
                )
                .as_bytes(),
            )?;
        }

        Command::new("mkisofs")
            .arg("-RJ")
            .arg("-V")
            .arg("cidata")
            .arg("-o")
            .arg(&user_data_img_path)
            .arg("-graft-points")
            .arg(format!("/={user_data_path}"))
            .arg(format!("/={meta_data_path}"))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| Error::UserDataCreationFailed(name.to_string()))?
            .wait()
            .map(|_| ())
            .map_err(|_| Error::UserDataCreationFailed(name.to_string()))?;
    }

    Result::Ok(())
}
