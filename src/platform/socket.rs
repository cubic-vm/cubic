use crate::error::Result;
use std::io::{Read, Write};

// A connection that can be cloned, so reads and writes can run side by side.
pub trait Socket: Read + Write + Send {
    fn try_clone(&self) -> Result<Box<dyn Socket>>;
}
