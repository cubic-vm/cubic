use crate::env::Environment;
use crate::image::ImageStore;
use crate::instance::InstanceStore;

pub struct Context {
    env: Environment,
    image_store: Box<dyn ImageStore>,
    instance_store: Box<dyn InstanceStore>,
}

impl Context {
    pub fn new(
        env: Environment,
        image_store: Box<dyn ImageStore>,
        instance_store: Box<dyn InstanceStore>,
    ) -> Self {
        Self {
            env,
            image_store,
            instance_store,
        }
    }

    pub fn get_env(&self) -> &Environment {
        &self.env
    }

    pub fn get_image_store(&self) -> &dyn ImageStore {
        self.image_store.as_ref()
    }

    pub fn get_instance_store(&self) -> &dyn InstanceStore {
        self.instance_store.as_ref()
    }
}
