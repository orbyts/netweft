use anyhow::Result;
use clap::Parser;
use netweft::cli::{Cli, Command, RenderCommand, ShowCommand};
use netweft::config::load::ConfigLoader;
use netweft::paths::NetweftPaths;
use netweft::plan::dns::resolve_dns_plan;
use netweft::plan::dns_access::derive_dns_access;
use netweft::render::bind::render_bind;

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
                ShowCommand::DnsAccess => {
                    let access = derive_dns_access(&bundle)?;
                    println!("DNS recursion access for '{}':", bundle.location.name);
                    println!("  IPv4:");
                    for network in access.ipv4 {
                        println!("    {network}");
                    }
                    println!("  IPv6:");
                    for network in access.ipv6 {
                        println!("    {network}");
                    }
                }
                ShowCommand::Dns => {
                    let plan = resolve_dns_plan(&bundle)?;
                    plan.print();
                }
            }
        }
        Command::Render { command } => {
            let bundle = ConfigLoader::new(&paths.config_dir).load(None)?;
            bundle.validate()?;

            match command {
                RenderCommand::Bind => {
                    let plan = resolve_dns_plan(&bundle)?;
                    let output = paths.generated_dir.join(&bundle.location.name).join("bind");
                    let rendered = render_bind(&plan, &output)?;
                    println!("Rendered BIND configuration: {}", rendered.display());
                }
            }
        }
    }

    Ok(())
}
