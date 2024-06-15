use crate::error::Error;
use crate::image::Image;
use crate::util;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

pub struct ImageFetcher {}

impl ImageFetcher {
    pub fn new() -> Self {
        ImageFetcher {}
    }

    pub fn fetch(&self, image: &Image, target_file: &str) -> Result<(), Error> {
        let temp_file = format!("{target_file}.tmp");
        if Path::new(&temp_file).exists() {
            util::remove_file(&temp_file)?;
        }

        if Path::new(&target_file).exists() {
            return Result::Ok(());
        }

        if Command::new("wget")
            .arg("-O")
            .arg(&temp_file)
            .arg(&image.url)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|_| Error::ImageDownloadFailed(image.to_id()))?
            .success()
        {
            fs::rename(temp_file, target_file).map_err(Error::Io)
        } else {
            Result::Err(Error::ImageDownloadFailed(image.to_id()))
        }
    }
}
