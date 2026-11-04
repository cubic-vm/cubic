use crate::commands::Context;
use crate::error::Result;
use crate::models::{Environment, Image};
use crate::platform::FileLock;
use crate::view::Spinner;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[must_use]
pub struct ResourceLock {
    _locks: Vec<FileLock>,
}

impl ResourceLock {
    // Sorted to avoid deadlocks
    pub fn acquire_instances(context: &Context, names: &[&str]) -> Result<Self> {
        let mut names = names.to_vec();
        names.sort();
        names.dedup();
        let locks = names
            .iter()
            .map(|name| {
                let path = Self::get_instance_path(context.get_env(), name);
                Self::acquire_file(context, &path, name)
            })
            .collect::<Result<_>>()?;
        Ok(Self { _locks: locks })
    }

    pub fn acquire_image(context: &Context, image: &Image) -> Result<Self> {
        let path = Self::get_image_path(context.get_env(), &image.to_file_name());
        let lock = Self::acquire_file(context, &path, &image.to_name())?;
        Ok(Self { _locks: vec![lock] })
    }

    pub fn try_acquire_instance(context: &Context, name: &str) -> Result<Option<Self>> {
        let path = Self::get_instance_path(context.get_env(), name);
        Self::try_acquire_file(context, &path)
    }

    pub fn try_acquire_image(context: &Context, file: &str) -> Result<Option<Self>> {
        let path = Self::get_image_path(context.get_env(), file);
        Self::try_acquire_file(context, &path)
    }

    fn acquire_file(context: &Context, path: &Path, name: &str) -> Result<FileLock> {
        let system = context.get_system();
        if let Some(lock) = system.lock_file(path, false)? {
            return Ok(lock);
        }
        let text = format!("Waiting for {name} lock");
        let _spinner = Spinner::new(Arc::clone(context.get_console()), text);
        let lock = system.lock_file(path, true)?;
        Ok(lock.expect("a waiting lock always returns the lock"))
    }

    fn try_acquire_file(context: &Context, path: &Path) -> Result<Option<Self>> {
        let lock = context.get_system().lock_file(path, false)?;
        Ok(lock.map(|lock| Self { _locks: vec![lock] }))
    }

    fn get_instance_path(env: &Environment, name: &str) -> PathBuf {
        env.get_instance_dir().join(format!("{name}.lock"))
    }

    fn get_image_path(env: &Environment, file: &str) -> PathBuf {
        let file = file.strip_suffix(".tmp").unwrap_or(file);
        env.get_image_dir().join(format!("{file}.lock"))
    }
}
