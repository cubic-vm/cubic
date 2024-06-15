use std::io;

pub enum Error {
    UnknownCommand,
    UnknownMachine(String),
    MachineNotStopped(String),
    Start(String),
    Stop(String),
    MachineAlreadyExists(String),
    Io(io::Error),
    UnknownImage(String),
    MissingSshKey,
    InvalidImageName(String),
    UnsetHomeVar,
    InvalidOption(String),
    CannotCopyFile(String, String),
    CannotCopyDir(String, String),
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
}
