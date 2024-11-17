use std::io;

#[derive(Debug)]
pub enum Error {
    UnknownCommand,
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
    CannotCopyFile(String, String),
    CannotCopyDir(String, String),
    CannotMoveDir(String, String),
    CannotOpenFile(String),
    CannotCreateFile(String),
    CannotCreateDir(String),
    CannotAccessDir(String),
    CannotWriteDir(String),
    CannotWriteFile(String),
    CannotRemoveFile(String),
    CannotParseFile(String),
    InvalidSshTarget(String),
    UserDataCreationFailed(String),
    CannotParseSize(String),
    CannotShrinkDisk(String),
    ImageDownloadFailed(String),
    GetCapacityFailed(String),
    CannotOpenTerminal(String),
    HostFwdRuleMalformed(String),
    CommandFailed(String),
}

pub fn print_error(error: Error) {
    print!("ERROR: ");
    match error {
        Error::UnknownCommand => println!("Unknown command"),
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
        Error::CannotCopyFile(from, to) => println!("Cannot copy file from '{from}' to '{to}'"),
        Error::CannotCopyDir(from, to) => println!("Cannot copy directory from '{from}' to '{to}'"),
        Error::CannotMoveDir(from, to) => println!("Cannot move directory from '{from}' to '{to}'"),
        Error::CannotCreateFile(path) => println!("Cannot create file '{path}'"),
        Error::CannotOpenFile(path) => println!("Cannot open file '{path}'"),
        Error::CannotCreateDir(path) => println!("Cannot create directory '{path}'"),
        Error::CannotAccessDir(path) => println!("Cannot access directory '{path}'"),
        Error::CannotWriteDir(path) => println!("Cannot write directory '{path}'"),
        Error::CannotWriteFile(path) => println!("Cannot write file '{path}'"),
        Error::CannotRemoveFile(path) => println!("Cannot write file '{path}'"),
        Error::CannotParseFile(path) => println!("Cannot parse file '{path}'"),
        Error::InvalidSshTarget(name) => println!("Invalid SSH target '{name}'"),
        Error::UserDataCreationFailed(name) => {
            println!("Failed to create user data for instance '{name}'")
        }
        Error::CannotParseSize(size) => println!("Invalid data size format '{size}'"),
        Error::CannotShrinkDisk(name) => {
            println!("Cannot shrink the disk of the instance '{name}'")
        }
        Error::ImageDownloadFailed(name) => println!("Failed to download image: '{name}'"),
        Error::GetCapacityFailed(path) => println!("Failed to get capacity from image: '{path}'"),
        Error::CannotOpenTerminal(path) => println!("Failed to open terminal from path: '{path}'"),
        Error::HostFwdRuleMalformed(rule) => println!("Host forwarding rule is malformed: {rule}"),
        Error::CommandFailed(message) => println!("{message}"),
    }
}
