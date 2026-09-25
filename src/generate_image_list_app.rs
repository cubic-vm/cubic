use crate::image::ImageFactory;
use crate::platform::{OsSystem, System};
use crate::view::Console;
use std::path::Path;
use std::sync::Arc;

pub struct GenerateImageListApp;

impl GenerateImageListApp {
    pub async fn run(output: &Path) -> i32 {
        let system: Arc<dyn System> = Arc::new(OsSystem::new());
        let console = &Console::new(Arc::clone(&system));
        let result = ImageFactory::get_images(console)
            .await
            .and_then(|images| ImageFactory::serialize(&images))
            .and_then(|list| system.write_file(output, list.as_bytes()));
        if let Err(error) = &result {
            console.error(&error.to_string());
        }

        console.flush();
        i32::from(result.is_err())
    }
}
