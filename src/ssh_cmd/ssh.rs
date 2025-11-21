use crate::view::Console;

pub trait Ssh {
    fn set_known_hosts_file(&mut self, path: Option<String>);
    fn set_private_keys(&mut self, private_keys: Vec<String>);
    fn set_args(&mut self, args: String);
    fn set_cmd(&mut self, cmd: Option<String>);
    fn shell(&mut self, console: &mut dyn Console, user: &str, port: u16, xforward: bool) -> bool;
    fn copy(&self, console: &mut dyn Console, root_dir: &str, from: &str, to: &str) -> bool;
}
