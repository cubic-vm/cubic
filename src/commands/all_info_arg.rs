use clap::Parser;

#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct AllInfoArg {
    /// Show all available information
    #[clap(short = 'a', long = "all", default_value_t = false)]
    pub value: bool,
}

impl From<bool> for AllInfoArg {
    fn from(value: bool) -> Self {
        Self { value }
    }
}
