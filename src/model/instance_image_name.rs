use crate::image::ImageName;
use crate::instance::InstanceName;
use std::fmt;
use std::str::FromStr;

#[derive(Clone)]
pub enum InstanceImageName {
    Image(ImageName),
    Instance(InstanceName),
}

impl FromStr for InstanceImageName {
    type Err = String;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        let image = ImageName::from_str(name);
        let instance = InstanceName::from_str(name);

        match (image, instance) {
            (Ok(i), _) => Ok(InstanceImageName::Image(i)),
            (_, Ok(i)) => Ok(InstanceImageName::Instance(i)),
            (Err(a), Err(b)) => Err(format!("{a}\n{b}")),
        }
    }
}

impl fmt::Display for InstanceImageName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            InstanceImageName::Image(image) => image.fmt(f),
            InstanceImageName::Instance(instance) => instance.fmt(f),
        }
    }
}
