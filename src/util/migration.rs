use crate::util;
use std::env;
use std::path::Path;

pub fn migrate() {
    // Move data from SNAP_USER_DATA to SNAP_COMMON_DATA
    if let Ok(user_data) = env::var("SNAP_USER_DATA") {
        if let Ok(user_common) = env::var("SNAP_USER_COMMON") {
            let from = format!("{user_data}/.local/share/cubic");
            let to = format!("{user_common}/cubic");
            if Path::new(&from).exists() && !Path::new(&to).exists() {
                util::move_dir(&from, &to).ok();
            }
        }
    }
}
