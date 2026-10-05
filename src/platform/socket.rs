use crate::error::Result;
use std::io::{Read, Write};
use std::pin::Pin;
use tokio::io::{AsyncRead, AsyncWrite};

// A connection that can be cloned, so reads and writes can run side by side.
pub trait Socket: Read + Write + Send {
    fn try_clone(&self) -> Result<Box<dyn Socket>>;
}

pub trait AsyncSocket: AsyncRead + AsyncWrite + Unpin + Send {}

impl<T: AsyncRead + AsyncWrite + Unpin + Send> AsyncSocket for T {}

pub type AsyncSocketFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Box<dyn AsyncSocket>>> + Send + 'a>>;
