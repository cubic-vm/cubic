use crate::view::TransferView;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, ReadBuf};

pub struct AsyncTransferView {
    pub read: Pin<Box<dyn AsyncRead + Unpin>>,
    pub size: usize,
    pub transfered: usize,
    pub view: TransferView,
}

impl AsyncTransferView {
    pub fn new(name: &str, read: Pin<Box<dyn AsyncRead + Unpin>>, size: usize) -> Self {
        Self {
            read,
            size,
            transfered: 0,
            view: TransferView::new(name),
        }
    }
}

impl AsyncRead for AsyncTransferView {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let is_done = self.transfered >= self.size;

        let before = buf.filled().len();
        let result = self.read.as_mut().poll_read(cx, buf);
        let after = buf.filled().len();
        self.transfered += after - before;
        let transfered = self.transfered;
        let size = self.size;

        if !is_done {
            self.view.update(transfered as u64, Some(size as u64));
        }
        result
    }
}
