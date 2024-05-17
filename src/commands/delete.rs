use crate::error::Error;
use crate::image::{is_image, ImageDao, ImageName};
use crate::machine::MachineDao;
use crate::util::confirm;

pub fn delete(
    image_dao: &ImageDao,
    machine_dao: &MachineDao,
    ids: &Vec<String>,
) -> Result<(), Error> {
    for id in ids {
        if is_image(id) {
            let name = ImageName::from_id(id)?;

            if !image_dao.exists(&name) {
                return Result::Err(Error::UnknownImage(name));
            }
        } else {
            if !machine_dao.exists(id) {
                return Result::Err(Error::UnknownMachine(id.clone()));
            }

            if machine_dao.is_running(&machine_dao.load(id)?) {
                return Result::Err(Error::MachineNotStopped(id.to_string()));
            }
        }
    }

    if confirm("Do you really want delete it? [y/n]: ") {
        for id in ids {
            if is_image(id) {
                let name = ImageName::from_id(id)?;
                image_dao.delete(&name)?;
                println!("Deleted image {id}");
            } else {
                machine_dao.delete(&machine_dao.load(id)?)?;
                println!("Deleted machine {id}");
            }
        }
    }

    Result::Ok(())
}
