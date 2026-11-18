use crate::error::Result;
use crate::platform::Socket;

pub trait Listener: Send + Sync {
    // Returns an error when no client is waiting.
    fn accept(&self) -> Result<Box<dyn Socket>>;
}
