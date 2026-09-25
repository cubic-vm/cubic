use crate::commands::CommandDispatcher;
use crate::platform::{OsSystem, System};
use crate::view::Console;
use clap::Parser;
use std::sync::Arc;

pub struct CubicApp;

impl CubicApp {
    pub async fn run() -> i32 {
        let os_system = Arc::new(OsSystem::new());
        let system: Arc<dyn System> = os_system.clone();
        let console = &Console::new(Arc::clone(&system));
        let result = CommandDispatcher::parse()
            .dispatch(Arc::clone(&system), console)
            .await;
        if let Err(error) = &result {
            console.error(&error.to_string());
        }
        let code = result.map_or(1, i32::from);

        console.flush();
        os_system.reap_children();
        code
    }
}
