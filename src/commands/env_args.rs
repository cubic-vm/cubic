use clap::Parser;

#[derive(Parser, Clone)]
pub struct EnvArgs {
    /// Environment variables to pass to the guest (KEY=VALUE or KEY to forward from host)
    #[clap(long = "env", value_name = "KEY[=VALUE]")]
    pub env_vars: Vec<String>,
}
