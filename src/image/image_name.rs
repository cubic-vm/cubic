use crate::error::Error;

#[derive(Clone)]
pub struct ImageName {
    pub vendor: String,
    pub image: String,
    pub arch: String,
}

impl ImageName {
    fn from(separator: char, id: &str) -> Result<Self, Error> {
        let mut tokens = id.split(separator);
        let vendor = tokens
            .next()
            .ok_or(Error::InvalidImageName(id.to_string()))?
            .to_string();
        let image = tokens
            .next()
            .ok_or(Error::InvalidImageName(id.to_string()))?
            .to_string();
        let arch = tokens.next().unwrap_or("amd64").to_string();
        Result::Ok(ImageName {
            vendor,
            image,
            arch,
        })
    }

    pub fn from_id(id: &str) -> Result<Self, Error> {
        Self::from(':', id)
    }

    pub fn from_file_name(id: &str) -> Result<Self, Error> {
        Self::from('_', id)
    }

    pub fn to_file_name(&self) -> String {
        format!("{}_{}_{}", self.vendor, self.image, self.arch)
    }
}
