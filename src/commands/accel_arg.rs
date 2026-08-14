use clap::{Parser, ValueEnum};

// Names the kind of acceleration, not the accelerator, since every platform
// has exactly one hardware accelerator.
#[derive(Clone, Copy, Debug, Default, PartialEq, ValueEnum)]
pub enum Accel {
    #[default]
    Auto,
    On,
    Off,
}

#[derive(Clone, Copy, Default, Parser)]
#[clap(verbatim_doc_comment)]
pub struct AccelArg {
    /// Set hardware acceleration
    #[clap(id = "accel", long = "accel", value_enum, default_value_t)]
    pub value: Accel,
}
