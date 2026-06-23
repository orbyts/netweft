use anyhow::Result;
use clap::Parser;
use netweft::adapter::AdapterContext;
use netweft::adapters::builtin_registry;
use netweft::cli::{Cli, Command, RenderCommand, ShowCommand};
use netweft::config::load::ConfigLoader;
use netweft::paths::NetweftPaths;
use netweft::plan::dns::resolve_dns_plan;
use netweft::plan::dns_access::derive_dns_access;
use netweft::plan::env::resolve_env_plan;

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
                ShowCommand::Env { host } => {
                    let plan = resolve_env_plan(&bundle, &paths, &host)?;
                    plan.print();
                }
            }
        }
        Command::Render { command } => {
            let bundle = ConfigLoader::new(&paths.config_dir).load(None)?;
            bundle.validate()?;
            let registry = builtin_registry()?;
            let context = AdapterContext::new(&bundle, &paths);

            match command {
                RenderCommand::Bind => {
                    let rendered = registry.get("bind")?.render(&context)?;
                    println!("Rendered BIND configuration: {}", rendered.root.display());
                }
                RenderCommand::Env { host } => {
                    let rendered = registry
                        .get("env")?
                        .render(&context.for_host(&host))?;
                    println!("Rendered host environment: {}", rendered.root.display());
                }
                RenderCommand::All { host } => {
                    let bind = registry.get("bind")?.render(&context)?;
                    let env = registry
                        .get("env")?
                        .render(&context.for_host(&host))?;
                    println!("Rendered BIND configuration: {}", bind.root.display());
                    println!("Rendered host environment: {}", env.root.display());
                }
            }
        }
    }

    Ok(())
}
