use crate::error::Error;
use crate::image::ImageName;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

pub struct ImageFetcher {}

fn get_url(name: &ImageName) -> Result<String, Error> {
    match name.vendor.as_ref() {
        "debian" => match name.image.as_ref() {
            "bullseye" => Result::Ok("https://cloud.debian.org/images/cloud/bullseye/latest/debian-11-generic-amd64.qcow2".to_string()),
            "bookworm" => Result::Ok("https://cloud.debian.org/images/cloud/bookworm/latest/debian-12-generic-amd64.qcow2".to_string()),
            _ => Result::Err(Error::UnknownImage(name.clone()))
        }

        "ubuntu" => Result::Ok(format!("https://cloud-images.ubuntu.com/{}/current/{}-server-cloudimg-amd64.img", &name.image, &name.image)),

        _ => Result::Err(Error::UnknownImage(name.clone()))
    }
}

impl ImageFetcher {
    pub fn new() -> Self {
        ImageFetcher {}
    }

    pub fn fetch(&self, name: &ImageName, target_file: &str) -> Result<(), Error> {
        let temp_file = format!("{target_file}.tmp");
        if Path::new(&target_file).exists() || Path::new(&temp_file).exists() {
            return Result::Ok(());
        }

        let url = get_url(name)?;

        Command::new("wget")
            .arg("-O")
            .arg(&temp_file)
            .arg(&url)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(Error::Io)?
            .wait()
            .map(|_| ())
            .map_err(Error::Io)?;

        fs::rename(temp_file, target_file).map_err(Error::Io)
    }
}
