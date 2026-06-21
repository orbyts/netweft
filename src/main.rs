use anyhow::Result;
use clap::Parser;
use netweft::cli::{Cli, Command, ShowCommand};
use netweft::config::load::ConfigLoader;
use netweft::paths::NetweftPaths;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let paths = NetweftPaths::resolve(cli.config_dir.as_deref())?;

    match cli.command {
        Command::Paths => {
            println!("{paths}");
        }
        Command::Validate { location } => {
            let bundle = ConfigLoader::new(&paths.config_dir).load(location.as_deref())?;
            let report = bundle.validate()?;

            println!(
                "Valid Netweft configuration for location '{}'",
                bundle.location.name
            );
            println!(
                "{} host(s), {} network(s), {} service(s), {} DNS zone(s)",
                bundle.inventory.hosts.len(),
                bundle.networks.networks.len(),
                bundle.services.services.len(),
                bundle.dns.zones.len()
            );

            for warning in report.warnings {
                eprintln!("warning: {warning}");
            }
        }
        Command::Show { command } => {
            let bundle = ConfigLoader::new(&paths.config_dir).load(None)?;
            match command {
                ShowCommand::Config => bundle.print_summary(),
                ShowCommand::Hosts => bundle.print_hosts(),
                ShowCommand::Networks => bundle.print_networks(),
                ShowCommand::Services => bundle.print_services(),
            }
        }
    }

    Ok(())
}
