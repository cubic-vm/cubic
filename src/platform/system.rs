use crate::platform::{FileSystem, Host, Network, Process, Terminal};

// The whole host surface, for callers that reach for more than one resource.
// Callers that need less are free to ask for a single resource trait instead.
pub trait System: FileSystem + Process + Network + Terminal + Host {}

impl<T> System for T where T: FileSystem + Process + Network + Terminal + Host {}
