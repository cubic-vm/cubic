use clap::{self, Parser, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
pub enum Iso9660 {
    System,
    Rust,
}

#[derive(Parser, Clone)]
#[clap(verbatim_doc_comment)]
pub struct Iso9660Arg {
    /// Switch for ISO9600 implementation
    #[clap(hide = true)]
    #[arg(value_enum, long = "iso9660", default_value_t = Iso9660::Rust)]
    pub value: Iso9660,
}
