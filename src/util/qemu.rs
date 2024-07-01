use crate::error::Error;
use crate::machine::{Machine, MountPoint, USER};
use crate::util;
use serde_json::Value::{self, Number};
use std::path::Path;
use std::process::{Command, Stdio};
use std::str;

fn run_qemu_info(path: &str) -> Result<Value, Error> {
    let out = Command::new("qemu-img")
        .arg("info")
        .arg("--output=json")
        .arg(path)
        .stdout(Stdio::piped())
        .output()
        .map_err(|_| Error::GetCapacityFailed(path.to_string()))?
        .stdout;

    let out_str = str::from_utf8(&out).map_err(|_| Error::GetCapacityFailed(path.to_string()))?;

    serde_json::from_str(out_str).map_err(|_| Error::GetCapacityFailed(path.to_string()))
}

pub fn get_disk_capacity(path: &str) -> Result<u64, Error> {
    let json: Value = run_qemu_info(path)?;

    match &json["virtual-size"] {
        Number(number) => number
            .as_u64()
            .ok_or(Error::GetCapacityFailed(path.to_string())),
        _ => Result::Err(Error::GetCapacityFailed(path.to_string())),
    }
}

pub fn get_disk_size(path: &str) -> Result<u64, Error> {
    let json: Value = run_qemu_info(path)?;

    match &json["actual-size"] {
        Number(number) => number
            .as_u64()
            .ok_or(Error::GetCapacityFailed(path.to_string())),
        _ => Result::Err(Error::GetCapacityFailed(path.to_string())),
    }
}

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

pub fn setup_cloud_init(machine: &Machine, dir: &str, force: bool) -> Result<(), Error> {
    let instance = &machine.name;

    let user_data_img_path = format!("{dir}/user-data.img");

    if force || !Path::new(&user_data_img_path).exists() {
        let metadata_path = format!("{dir}/metadata.yaml");
        let user_data_path = format!("{dir}/user-data.yaml");

        util::create_dir(dir)?;

        if force || !Path::new(&metadata_path).exists() {
            util::write_file(
                &metadata_path,
                format!("instance-id: {instance}\nlocal-hostname: {instance}\n").as_bytes(),
            )?;
        }

        let mut bootcmds = String::new();
        for (index, MountPoint { guest, .. }) in machine.mounts.iter().enumerate() {
            bootcmds += &format!("  - mount -t 9p cubic{index} {guest}\n");
        }

        if force || !Path::new(&user_data_path).exists() {
            let ssh_pk = util::get_ssh_pub_keys()?.join("\n\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}- ");
            util::write_file(
                &user_data_path,
                format!(
                    "\
                    #cloud-config\n\
                    users:\n\
                    \u{20}\u{20}- name: {USER}\n\
                    \u{20}\u{20}\u{20}\u{20}ssh-authorized-keys:\n\
                    \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}- {ssh_pk}\n\
                    \u{20}\u{20}\u{20}\u{20}shell: /bin/bash\n\
                    \u{20}\u{20}\u{20}\u{20}sudo: ALL=(ALL) NOPASSWD:ALL\n\
                    bootcmd:\n\
                    {bootcmds}"
                )
                .as_bytes(),
            )?;
        }

        Command::new("cloud-localds")
            .arg(&user_data_img_path)
            .arg(user_data_path)
            .arg(metadata_path)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|_| Error::UserDataCreationFailed(instance.to_string()))?
            .wait()
            .map(|_| ())
            .map_err(|_| Error::UserDataCreationFailed(instance.to_string()))?;
    }

    Result::Ok(())
}
