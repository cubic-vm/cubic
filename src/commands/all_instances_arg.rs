use clap::Parser;

#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct AllInstancesArg {
    /// Apply to all virtual machine instances
    #[clap(short = 'a', long = "all", default_value_t = false)]
    pub value: bool,
}

impl From<bool> for AllInstancesArg {
    fn from(value: bool) -> Self {
        Self { value }
    }
}
