use std::io;

#[derive(Debug)]
pub enum Error {
    UnknownCommand,
    InvalidArgument(String),
    UnknownArch(String),
    UnknownInstance(String),
    InstanceIsRunning(String),
    InstanceNotStopped(String),
    InstanceAlreadyExists(String),
    Io(io::Error),
    FS(String),
    UnknownImage(String),
    InvalidImageName(String),
    UnsetEnvVar(String),
    CannotParseFile(String),
    InvalidSshTarget(String),
    CannotParseSize(String),
    CannotShrinkDisk(String),
    CannotOpenTerminal(String),
    HostFwdRuleMalformed(String),
    SystemCommandFailed(String, String),
    Web(reqwest::Error),
    SerdeJson(serde_json::Error),
    SerdeYaml(serde_yaml::Error),
}

pub fn print_error(error: Error) {
    print!("ERROR: ");
    match error {
        Error::UnknownCommand => println!("Unknown command"),
        Error::InvalidArgument(err) => println!("Argument error: {err}"),
        Error::UnknownArch(name) => println!("Unknown architecture: '{name}'"),
        Error::UnknownInstance(instance) => println!("Unknown instance '{instance}'"),
        Error::InstanceIsRunning(name) => println!("Instance '{name}' is already runing"),
        Error::InstanceNotStopped(name) => println!("Instance '{name}' is not stopped"),
        Error::InstanceAlreadyExists(id) => println!("Instance with name '{id}' already exists"),
        Error::Io(e) => println!("{}", e),
        Error::FS(e) => println!("{}", e),
        Error::UnknownImage(name) => println!("Unknown image name {name}"),
        Error::InvalidImageName(name) => println!("Invalid image name: {name}"),
        Error::UnsetEnvVar(var) => println!("Environment variable '{var}' is not set"),
        Error::CannotParseFile(path) => println!("Cannot parse file '{path}'"),
        Error::InvalidSshTarget(name) => println!("Invalid SSH target '{name}'"),
        Error::CannotParseSize(size) => println!("Invalid data size format '{size}'"),
        Error::CannotShrinkDisk(name) => {
            println!("Cannot shrink the disk of the instance '{name}'")
        }
        Error::CannotOpenTerminal(path) => println!("Failed to open terminal from path: '{path}'"),
        Error::HostFwdRuleMalformed(rule) => println!("Host forwarding rule is malformed: {rule}"),
        Error::SystemCommandFailed(cmd, stderr) => {
            println!(
                "System command execution failed\n{cmd}\n\nReason: {}",
                stderr.trim()
            )
        }
        Error::SerdeJson(err) => println!("[JSON] {err}"),
        Error::SerdeYaml(err) => println!("[YAML] {err}"),
        Error::Web(e) => println!("{e}"),
    }
}
