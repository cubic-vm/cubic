use crate::error::Error;
use crate::image::{is_image, ImageDao};
use crate::machine::MachineDao;
use crate::util::confirm;

pub fn delete(
    image_dao: &ImageDao,
    machine_dao: &MachineDao,
    ids: &Vec<String>,
) -> Result<(), Error> {
    for id in ids {
        if is_image(id) {
            let image = image_dao.get(id)?;

            if !image_dao.exists(&image) {
                return Result::Err(Error::UnknownImage(image.to_id()));
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
                let image = image_dao.get(id)?;
                image_dao.delete(&image)?;
                println!("Deleted image {id}");
            } else {
                machine_dao.delete(&machine_dao.load(id)?)?;
                println!("Deleted machine {id}");
            }
        }
    }

    Result::Ok(())
}
