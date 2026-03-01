use crate::error::Error;
use russh_sftp::{self, client::SftpSession};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::{self, fs};
use tokio::io::{AsyncRead, AsyncWrite};

#[derive(Clone)]
pub struct SftpPath {
    pub sftp: Option<Rc<SftpSession>>,
    pub path: PathBuf,
}

impl SftpPath {
    pub fn name(&self) -> String {
        self.path.file_name().unwrap().to_str().unwrap().to_string()
    }

    pub fn to_str(&self) -> &str {
        self.path.to_str().unwrap()
    }

    pub async fn exists(&self) -> bool {
        match &self.sftp {
            None => self.path.exists(),
            Some(sftp) => sftp.try_exists(self.to_str()).await.unwrap(),
        }
    }

    pub async fn is_file(&self) -> Result<bool, Error> {
        match &self.sftp {
            None => Ok(self.path.is_file()),
            Some(sftp) => sftp
                .metadata(self.to_str())
                .await
                .map_err(|_| Error::InvalidPath(self.to_str().to_string()))
                .map(|metadata| metadata.file_type().is_file()),
        }
    }

    pub async fn is_dir(&self) -> Result<bool, Error> {
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
        let mut path = self.path.clone();
        path.push(name);
        Self {
            sftp: self.sftp.clone(),
            path,
        }
    }

    pub async fn open_file(&self) -> Box<dyn AsyncRead + Unpin> {
        match &self.sftp {
            None => Box::new(tokio::fs::File::open(self.path.clone()).await.unwrap()),
            Some(sftp) => Box::new(sftp.open(self.to_str()).await.unwrap()),
        }
    }

    pub async fn create_file(&self) -> Box<dyn AsyncWrite + Unpin> {
        match &self.sftp {
            None => Box::new(tokio::fs::File::create(self.path.clone()).await.unwrap()),
            Some(sftp) => Box::new(sftp.create(self.to_str()).await.unwrap()),
        }
    }

    pub async fn write_file(&self, mut content: Box<dyn AsyncRead + Unpin>) {
        tokio::io::copy(content.as_mut(), &mut self.create_file().await)
            .await
            .unwrap();
    }

    pub async fn read_dir(&self) -> Vec<SftpPath> {
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
                for entry in sftp.read_dir(self.to_str()).await.unwrap() {
                    children.push(Self {
                        sftp: Some(sftp.clone()),
                        path: Path::new(&format!("{}/{}", self.to_str(), entry.file_name()))
                            .to_path_buf(),
                    });
                }
            }
        }
        children
    }

    pub async fn create_path(&self) {
        match &self.sftp {
            None => fs::create_dir(self.path.clone()).unwrap(),
            Some(sftp) => sftp.create_dir(self.to_str()).await.unwrap(),
        }
    }

    pub async fn recursive_copy(&self, target: SftpPath) -> Result<(), Error> {
        if self.is_file().await? {
            let reader = self.open_file().await;
            if target.exists().await && target.is_dir().await? {
                target.append(&self.name()).write_file(reader).await;
            } else {
                target.write_file(reader).await;
            }
        } else if self.is_dir().await? {
            let target_dir = target.append(&self.name());
            target_dir.create_path().await;
            for entry in self.read_dir().await {
                Box::pin(entry.recursive_copy(target_dir.clone())).await?;
            }
        }

        Ok(())
    }

    pub async fn copy(&self, target: SftpPath) -> Result<(), Error> {
        if target.exists().await || self.is_file().await? {
            self.recursive_copy(target).await?;
        } else if self.is_dir().await? {
            target.create_path().await;
            for entry in self.read_dir().await {
                entry.recursive_copy(target.clone()).await?;
            }
        }

        Ok(())
    }
}
