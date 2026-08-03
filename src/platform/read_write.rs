use std::io::{Read, Write};

// A stream that can be both read and written, so a connection can be handed
// out as a trait object without naming the transport behind it.
pub trait ReadWrite: Read + Write {}
impl<T: Read + Write> ReadWrite for T {}
