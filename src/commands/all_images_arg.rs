use clap::Parser;

#[derive(Parser)]
#[clap(verbatim_doc_comment)]
pub struct AllImagesArg {
    /// Show all images
    #[clap(short = 'a', long = "all", default_value_t = false)]
    pub value: bool,
}
