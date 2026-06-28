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

    /// Inspect typed or derived configuration objects.
    Show {
        #[command(subcommand)]
        command: ShowCommand,
    },

    /// Generate configuration artifacts without deploying them.
    Render {
        #[command(subcommand)]
        command: RenderCommand,
    },

    /// Inspect adapters compiled into this Netweft binary.
    Adapters {
        #[command(subcommand)]
        command: AdapterCommand,
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
    /// Show networks automatically allowed to use DNS recursion.
    DnsAccess,
    /// Show the resolved DNS plan without generating files.
    Dns,
    /// Show the provider-neutral reverse-proxy plan.
    Proxy,
    /// Show the generated environment plan for a host.
    Env {
        #[arg(long)]
        host: String,
    },
    /// Show the resolved operating-system network plan for a host.
    HostNetwork {
        #[arg(long)]
        host: String,
    },
    /// Show resolved Proxmox VM/LXC guest identities and addresses.
    Guests,
    /// Show resolved network mounts for a host.
    NetworkMounts {
        #[arg(long)]
        host: String,
    },
    /// Show resolved NAS-side network permissions.
    NasPermissions {
        #[arg(long)]
        nas: Option<String>,
    },
    /// Show resolved Proxmox storage for a host.
    ProxmoxStorage {
        #[arg(long)]
        host: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum RenderCommand {
    /// Render a complete BIND configuration for the active location.
    Bind,
    /// Render native Nginx reverse-proxy configuration.
    Nginx {
        /// Restrict rendering to proxies deployed on this host.
        #[arg(long)]
        host: Option<String>,
        /// Run nginx -t after rendering.
        #[arg(long)]
        check: bool,
        /// Nginx executable used by --check.
        #[arg(long, default_value = "nginx")]
        nginx: PathBuf,
    },
    /// Render Docker Compose and shell environment files for a host.
    Env {
        #[arg(long)]
        host: String,
    },
    /// Render Proxmox ifupdown2 host networking.
    Proxmox {
        #[arg(long)]
        host: String,
    },
    /// Render systemd network mount units and service dependencies.
    SystemdMounts {
        #[arg(long)]
        host: String,
    },
    /// Render a Synology NFS permission action plan.
    SynologyNfsPermissions {
        #[arg(long)]
        nas: String,
    },
    /// Render Proxmox storage configuration and deployment scripts.
    ProxmoxStorage {
        #[arg(long)]
        host: String,
    },
    /// Render all artifacts for a host.
    All {
        #[arg(long)]
        host: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum AdapterCommand {
    /// List adapters and their capabilities.
    List,
}
