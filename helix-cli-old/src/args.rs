
#[derive(Debug, Parser)]
#[clap(name = NAME, version = VERSION, author = AUTHORS)]
pub struct HelixCLI {
    #[clap(subcommand)]
    pub command: CommandType,
}

#[derive(Debug, Subcommand, Clone, ValueEnum)]
#[clap(name = "output")]
pub enum OutputLanguage {
    #[clap(name = "rust", alias = "rs")]
    Rust,
    #[clap(name = "typescript", alias = "ts")]
    TypeScript,
}

impl PartialEq for OutputLanguage {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (OutputLanguage::TypeScript, OutputLanguage::TypeScript) => true,
            (OutputLanguage::Rust, OutputLanguage::Rust) => true,
            _ => false,
        }
    }
}
