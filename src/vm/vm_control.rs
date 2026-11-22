use crate::error::Result;

pub trait VmControl {
    fn stop(&mut self) -> Result<()>;
    fn quit(&mut self) -> Result<()>;
}
