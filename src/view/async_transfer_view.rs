use crate::view::TransferView;
use std::io::{self, Write};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub trait AsyncFile: AsyncRead + AsyncWrite + Unpin {}

impl<T: AsyncRead + AsyncWrite + Unpin> AsyncFile for T {}

pub struct AsyncTransferView {
    pub read: Box<dyn AsyncFile>,
    pub view: TransferView,
}

impl AsyncTransferView {
    pub fn new(view: TransferView, read: Box<dyn AsyncFile>) -> Self {
        Self { read, view }
    }
}

impl AsyncRead for AsyncTransferView {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let before = buf.filled().len();
        let result = Pin::new(&mut self.read).poll_read(cx, buf);
        self.view.write_all(&buf.filled()[before..])?;
        result
    }
}
