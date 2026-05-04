use clap::Parser;

#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct YesArg {
    /// Answer all questions with yes
    #[clap(short = 'y', long = "yes", default_value_t = false)]
    pub value: bool,
}
