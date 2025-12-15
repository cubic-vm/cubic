use crate::error::Error;
use crate::instance::Instance;
use std::io::Read;

pub trait InstanceDeserializer {
    fn deserialize(&self, name: &str, reader: &mut dyn Read) -> Result<Instance, Error>;
}
