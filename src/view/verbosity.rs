#[derive(PartialEq, Clone, Copy)]
pub enum Verbosity {
    Verbose,
    Normal,
    Quiet,
}

impl Verbosity {
    pub fn new(verbose: bool, quiet: bool) -> Self {
        if quiet {
            Verbosity::Quiet
        } else if verbose {
            Verbosity::Verbose
        } else {
            Verbosity::Normal
        }
    }

    pub fn is_verbose(&self) -> bool {
        *self == Verbosity::Verbose
    }

    pub fn is_quiet(&self) -> bool {
        *self == Verbosity::Quiet
    }
}
