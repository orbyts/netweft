use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "netweft",
    version,
    about = "Deterministic network planning and configuration generation"
)]
pub struct Cli {
    /// Override the Netweft source configuration directory.
    #[arg(long, global = true, env = "NETWEFT_CONFIG_DIR")]
    pub config_dir: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Print the resolved Netweft filesystem paths.
    Paths,

    /// Load and validate the configuration.
    Validate {
        /// Override the active location from netweft.toml.
        #[arg(long)]
        location: Option<String>,
    },

    /// Inspect typed configuration objects.
    Show {
        #[command(subcommand)]
        command: ShowCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum ShowCommand {
    /// Print a summary of the selected configuration.
    Config,
    /// List stable hosts.
    Hosts,
    /// List stable logical networks.
    Networks,
    /// List services.
    Services,
}
