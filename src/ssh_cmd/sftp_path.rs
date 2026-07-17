use crate::error::{Error, Result};
use crate::view::{AsyncTransferView, Console, TransferView};
use russh_sftp::{self, client::SftpSession};
use std::cmp::max;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::{self, fs};
use tokio::io::{AsyncRead, AsyncWrite};

// SFTP always requires forward-slash paths, regardless of the host OS.
// PathBuf::push/join would insert the compile-target's native separator
// (backslash on Windows), corrupting remote paths, so remote joins are
// built as plain strings instead.
pub fn join_path_segment(base: &str, name: &str, is_remote: bool) -> String {
    if is_remote {
        format!("{}/{}", base.trim_end_matches('/'), name)
    } else {
        PathBuf::from(base)
            .join(name)
            .to_string_lossy()
            .into_owned()
    }
}

#[derive(Clone)]
pub struct SftpPath {
    pub sftp: Option<Rc<SftpSession>>,
    pub path: PathBuf,
}

impl SftpPath {
    pub fn name(&self) -> Result<String> {
        self.path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .ok_or_else(|| Error::InvalidPath(self.path.display().to_string()))
    }

    pub fn to_str(&self) -> &str {
        self.path.to_str().unwrap()
    }

    pub async fn exists(&self) -> Result<bool> {
        match &self.sftp {
            None => Ok(self.path.exists()),
            Some(sftp) => sftp
                .try_exists(self.to_str())
                .await
                .map_err(|e| Error::Sftp(e.to_string())),
        }
    }

    pub async fn get_file_size(&self) -> Result<usize> {
        match &self.sftp {
            None => self
                .path
                .metadata()
                .map_err(|_| Error::InvalidPath(self.to_str().to_string()))
                .map(|metadata| metadata.len() as usize),
            Some(sftp) => sftp
                .metadata(self.to_str())
                .await
                .map_err(|_| Error::InvalidPath(self.to_str().to_string()))
                .and_then(|metadata| {
                    metadata
                        .size
                        .map(|size| size as usize)
                        .ok_or_else(|| Error::InvalidPath(self.to_str().to_string()))
                }),
        }
    }

    pub async fn is_file(&self) -> Result<bool> {
        match &self.sftp {
            None => Ok(self.path.is_file()),
            Some(sftp) => sftp
                .metadata(self.to_str())
                .await
                .map_err(|_| Error::InvalidPath(self.to_str().to_string()))
                .map(|metadata| metadata.file_type().is_file()),
        }
    }

    pub async fn is_dir(&self) -> Result<bool> {
        match &self.sftp {
            None => Ok(self.path.is_dir()),
            Some(sftp) => sftp
                .metadata(self.to_str())
                .await
                .map_err(|_| Error::InvalidPath(self.to_str().to_string()))
                .map(|metadata| metadata.file_type().is_dir()),
        }
    }

    pub fn append(&self, name: &str) -> Self {
        let joined = join_path_segment(self.to_str(), name, self.sftp.is_some());
        Self {
            sftp: self.sftp.clone(),
            path: PathBuf::from(joined),
        }
    }

    pub async fn open_file(&self) -> Result<Box<dyn AsyncRead + Unpin>> {
        match &self.sftp {
            None => tokio::fs::File::open(self.path.clone())
                .await
                .map(|f| Box::new(f) as Box<dyn AsyncRead + Unpin>)
                .map_err(Error::Io),
            Some(sftp) => sftp
                .open(self.to_str())
                .await
                .map(|f| Box::new(f) as Box<dyn AsyncRead + Unpin>)
                .map_err(|e| Error::Sftp(e.to_string())),
        }
    }

    pub async fn create_file(&self) -> Result<Box<dyn AsyncWrite + Unpin>> {
        match &self.sftp {
            None => tokio::fs::File::create(self.path.clone())
                .await
                .map(|f| Box::new(f) as Box<dyn AsyncWrite + Unpin>)
                .map_err(Error::Io),
            Some(sftp) => sftp
                .create(self.to_str())
                .await
                .map(|f| Box::new(f) as Box<dyn AsyncWrite + Unpin>)
                .map_err(|e| Error::Sftp(e.to_string())),
        }
    }

    pub async fn write_file(
        &self,
        console: &mut dyn Console,
        name: &str,
        size: usize,
        content: Box<dyn AsyncRead + Unpin>,
    ) -> Result<()> {
        let name = &format!("{:30}", &name[max(30, name.len()) - 30..name.len()]);
        let view = Arc::new(Mutex::new(TransferView::new(name)));
        console.play(view.clone());
        let read = &mut AsyncTransferView::new(view, std::pin::Pin::new(content), size);
        let result = tokio::io::copy(read, &mut self.create_file().await?)
            .await
            .map(|_| ())
            .map_err(Error::Io);
        console.stop();
        result
    }

    pub async fn read_dir(&self) -> Result<Vec<SftpPath>> {
        let mut children = Vec::new();
        match &self.sftp {
            None => {
                if let Ok(dir) = self.path.read_dir() {
                    for entry in dir.flatten() {
                        children.push(Self {
                            sftp: None,
                            path: entry.path(),
                        });
                    }
                }
            }
            Some(sftp) => {
                for entry in sftp
                    .read_dir(self.to_str())
                    .await
                    .map_err(|e| Error::Sftp(e.to_string()))?
                {
                    children.push(Self {
                        sftp: Some(sftp.clone()),
                        path: Path::new(&format!("{}/{}", self.to_str(), entry.file_name()))
                            .to_path_buf(),
                    });
                }
            }
        }
        Ok(children)
    }

    pub async fn create_path(&self) -> Result<()> {
        match &self.sftp {
            None => fs::create_dir(self.path.clone()).map_err(Error::Io),
            Some(sftp) => sftp
                .create_dir(self.to_str())
                .await
                .map_err(|e| Error::Sftp(e.to_string())),
        }
    }

    pub async fn recursive_copy(&self, console: &mut dyn Console, target: SftpPath) -> Result<()> {
        if self.is_file().await? {
            let name = &self.path.display().to_string();
            let size = self.get_file_size().await?;
            let reader = self.open_file().await?;
            if target.exists().await? && target.is_dir().await? {
                target
                    .append(&self.name()?)
                    .write_file(console, name, size, reader)
                    .await?;
            } else {
                target.write_file(console, name, size, reader).await?;
            }
        } else if self.is_dir().await? {
            let target_dir = target.append(&self.name()?);
            target_dir.create_path().await?;
            for entry in self.read_dir().await? {
                Box::pin(entry.recursive_copy(console, target_dir.clone())).await?;
            }
        }

        Ok(())
    }

    pub async fn copy(&self, console: &mut dyn Console, target: SftpPath) -> Result<()> {
        if !self.exists().await? {
            return Err(Error::InvalidPath(self.path.display().to_string()));
        }

        if target.exists().await? || self.is_file().await? {
            self.recursive_copy(console, target).await?;
        } else if self.is_dir().await? {
            target.create_path().await?;
            for entry in self.read_dir().await? {
                entry.recursive_copy(console, target.clone()).await?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_path_segment_remote_uses_forward_slash() {
        assert_eq!(
            join_path_segment("/home/cubic/dir1", "dir2", true),
            "/home/cubic/dir1/dir2"
        );
    }

    #[test]
    fn test_join_path_segment_remote_nested_appends_contain_no_backslash() {
        let first = join_path_segment("/home/cubic", "dir1", true);
        let second = join_path_segment(&first, "file.txt", true);
        assert_eq!(second, "/home/cubic/dir1/file.txt");
        assert!(!second.contains('\\'));
    }

    #[test]
    fn test_join_path_segment_remote_trailing_slash_base() {
        assert_eq!(
            join_path_segment("/home/cubic/", "dir1", true),
            "/home/cubic/dir1"
        );
    }

    #[test]
    fn test_join_path_segment_local_matches_native_pathbuf_join() {
        let result = join_path_segment("base", "child", false);
        let expected = PathBuf::from("base")
            .join("child")
            .to_string_lossy()
            .into_owned();
        assert_eq!(result, expected);
    }
}
