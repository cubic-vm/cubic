use std::time::{SystemTime, UNIX_EPOCH};

const PORT_MIN: u128 = 1024;
const PORT_MAX: u128 = 65535;

pub fn generate_random_ssh_port() -> u16 {
    (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        % (PORT_MAX - PORT_MIN)
        + PORT_MIN) as u16
}
