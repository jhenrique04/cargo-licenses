use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "A tool to report licenses of direct dependencies from Cargo.toml (async)."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Generate a license report (Markdown or JSON)
    Generate {
        #[arg(long, value_enum, default_value = "md")]
        format: OutputFormat,

        #[arg(long, default_value_t = false)]
        dev: bool,

        #[arg(long, default_value_t = false)]
        build: bool,

        #[arg(long)]
        skip_optional: bool,
    },

    /// Just list direct dependencies
    List {
        #[arg(long, default_value_t = false)]
        dev: bool,
        #[arg(long, default_value_t = false)]
        build: bool,
        #[arg(long, default_value_t = false)]
        skip_optional: bool,
    },

    /// Check licenses against a deny/allow list
    Check {
        /// Disallowed license strings (can appear multiple times)
        #[arg(long)]
        deny: Vec<String>,

        /// Allowed license strings (can appear multiple times)
        #[arg(long)]
        allow: Vec<String>,

        #[arg(long, default_value_t = false)]
        dev: bool,
        #[arg(long, default_value_t = false)]
        build: bool,
        #[arg(long)]
        skip_optional: bool,
    },

    Version,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Md,
    Json,
}
