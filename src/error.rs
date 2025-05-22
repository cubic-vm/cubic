use std::io;

#[derive(Debug)]
pub enum Error {
    UnknownCommand,
    UnknownArch(String),
    UnknownInstance(String),
    InstanceIsRunning(String),
    InstanceNotStopped(String),
    Start(String),
    InstanceAlreadyExists(String),
    Io(io::Error),
    UnknownImage(String),
    MissingSshKey,
    InvalidImageName(String),
    UnsetEnvVar(String),
    CannotAccessDir(String),
    CannotWriteDir(String),
    CannotParseFile(String),
    InvalidSshTarget(String),
    UserDataCreationFailed(String),
    CannotParseSize(String),
    CannotShrinkDisk(String),
    GetCapacityFailed(String),
    CannotOpenTerminal(String),
    HostFwdRuleMalformed(String),
    CommandFailed(String),
    Web(reqwest::Error),
    SerdeJson(serde_json::Error),
    SerdeYaml(serde_yaml::Error),
    MissingQemuGA,
    ExecFailed,
}

pub fn print_error(error: Error) {
    print!("ERROR: ");
    match error {
        Error::UnknownCommand => println!("Unknown command"),
        Error::UnknownArch(name) => println!("Unknown architecture: '{name}'"),
        Error::UnknownInstance(instance) => println!("Unknown instance '{instance}'"),
        Error::InstanceIsRunning(name) => println!("Instance '{name}' is already runing"),
        Error::InstanceNotStopped(name) => println!("Instance '{name}' is not stopped"),
        Error::Start(instance) => println!("Failed to start instance '{instance}'"),
        Error::InstanceAlreadyExists(id) => println!("Instance with name '{id}' already exists"),
        Error::Io(e) => println!("{}", e),
        Error::UnknownImage(name) => println!("Unknown image name {name}"),
        Error::MissingSshKey => print!(
            "No SSH keys found. Please try the following:\n\
- Check if cubic has read access to $HOME/.ssh
  - Snap users must grant access with: `sudo snap connect cubic:ssh-keys`\n\
- Check if you have a ssh key in $HOME/.ssh
  - You can generate one with `ssh-keygen`\n\
- Use `cubic sh myinstance` instead\n"
        ),
        Error::InvalidImageName(name) => println!("Invalid image name: {name}"),
        Error::UnsetEnvVar(var) => println!("Environment variable '{var}' is not set"),
        Error::CannotAccessDir(path) => println!("Cannot access directory '{path}'"),
        Error::CannotWriteDir(path) => println!("Cannot write directory '{path}'"),
        Error::CannotParseFile(path) => println!("Cannot parse file '{path}'"),
        Error::InvalidSshTarget(name) => println!("Invalid SSH target '{name}'"),
        Error::UserDataCreationFailed(name) => {
            println!("Failed to create user data for instance '{name}'")
        }
        Error::CannotParseSize(size) => println!("Invalid data size format '{size}'"),
        Error::CannotShrinkDisk(name) => {
            println!("Cannot shrink the disk of the instance '{name}'")
        }
        Error::GetCapacityFailed(path) => println!("Failed to get capacity from image: '{path}'"),
        Error::CannotOpenTerminal(path) => println!("Failed to open terminal from path: '{path}'"),
        Error::HostFwdRuleMalformed(rule) => println!("Host forwarding rule is malformed: {rule}"),
        Error::CommandFailed(message) => println!("{message}"),
        Error::SerdeJson(err) => println!("[JSON] {err}"),
        Error::SerdeYaml(err) => println!("[YAML] {err}"),
        Error::MissingQemuGA => println!("Cannot access QEMU guest agent. Please install qemu-guest-agent in the virtual machine instance."),
        Error::ExecFailed => println!("Failed to execute command in virtual machine instance."),
        Error::Web(e) => println!("{e}"),
    }
}
