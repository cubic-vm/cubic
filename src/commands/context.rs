use crate::image::ImageStore;
use crate::instance::InstanceStore;
use crate::models::Environment;
use crate::platform::System;
use std::rc::Rc;

pub struct Context {
    system: Rc<dyn System>,
    env: Environment,
    image_store: Box<dyn ImageStore>,
    instance_store: Box<dyn InstanceStore>,
}

impl Context {
    pub fn new(
        system: Rc<dyn System>,
        env: Environment,
        image_store: Box<dyn ImageStore>,
        instance_store: Box<dyn InstanceStore>,
    ) -> Self {
        Self {
            system,
            env,
            image_store,
            instance_store,
        }
    }

    pub fn get_system(&self) -> &dyn System {
        self.system.as_ref()
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
