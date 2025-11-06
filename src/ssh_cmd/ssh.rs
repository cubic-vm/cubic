pub trait Ssh {
    fn set_known_hosts_file(&mut self, path: Option<String>);
    fn set_private_keys(&mut self, private_keys: Vec<String>);
    fn set_user(&mut self, user: String);
    fn set_port(&mut self, port: Option<u16>);
    fn set_args(&mut self, args: String);
    fn set_xforward(&mut self, xforward: bool);
    fn set_cmd(&mut self, cmd: Option<String>);
    fn connect(&mut self) -> bool;
    fn get_command(&mut self) -> String;
}
