use crate::error::Result;
use reqwest::Response;
use std::io::Write;

pub struct Download {
    pub(super) resp: Response,
}

impl Download {
    pub fn size(&self) -> Option<u64> {
        self.resp.content_length()
    }

    pub async fn text(self) -> Result<String> {
        Ok(self.resp.text().await?)
    }

    pub async fn write_to(mut self, writers: &mut [&mut dyn Write]) -> Result<()> {
        while let Some(chunk) = self.resp.chunk().await? {
            for writer in writers.iter_mut() {
                writer.write_all(&chunk)?;
            }
        }

        for writer in writers.iter_mut() {
            writer.flush()?;
        }
        Ok(())
    }
}
