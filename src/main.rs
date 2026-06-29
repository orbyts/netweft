use anyhow::Result;
use clap::Parser;
use netweft::adapter::AdapterContext;
use netweft::adapters::builtin_registry;
use netweft::adapters::nginx::validate_nginx_config;
use netweft::cli::{AdapterCommand, Cli, Command, RenderCommand, ShowCommand};
use netweft::config::load::ConfigLoader;
use netweft::paths::NetweftPaths;
use netweft::plan::dns::resolve_dns_plan;
use netweft::plan::dns_access::derive_dns_access;
use netweft::plan::docker::resolve_docker_plan;
use netweft::plan::env::resolve_env_plan;
use netweft::plan::guest::resolve_guest_plan;
use netweft::plan::host_network::resolve_host_network_plan;
use netweft::plan::nas_permission::resolve_nas_permission_plan;
use netweft::plan::network_mount::resolve_network_mount_plan;
use netweft::plan::os_network::resolve_os_network_plan;
use netweft::plan::proxmox_sdn::resolve_proxmox_sdn_plan;
use netweft::plan::proxmox_storage::resolve_proxmox_storage_plan;
use netweft::plan::proxy::resolve_proxy_plan;
use netweft::plan::ssh::resolve_ssh_plan;
use netweft::resolve::ResolvedPlan;

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
                ShowCommand::Docker { host } => {
                    resolve_docker_plan(&bundle, &host)?.print();
                }
                ShowCommand::Ssh { client } => {
                    resolve_ssh_plan(&bundle, &client)?.print();
                }
                ShowCommand::Proxy => {
                    let plan = resolve_proxy_plan(&bundle)?;
                    plan.print();
                }
                ShowCommand::Env { host } => {
                    let plan = resolve_env_plan(&bundle, &paths, &host)?;
                    plan.print();
                }
                ShowCommand::HostNetwork { host } => {
                    let plan = resolve_host_network_plan(&bundle, &host)?;
                    plan.print();
                }
                ShowCommand::OsNetwork { host } => {
                    resolve_os_network_plan(&bundle, &host)?.print();
                }
                ShowCommand::ProxmoxSdn { host } => {
                    resolve_proxmox_sdn_plan(&bundle, &host)?.print();
                }
                ShowCommand::Guests => {
                    resolve_guest_plan(&bundle)?.print();
                }
                ShowCommand::NetworkMounts { host } => {
                    resolve_network_mount_plan(&bundle, &host)?.print();
                }
                ShowCommand::NasPermissions { nas } => {
                    resolve_nas_permission_plan(&bundle, nas.as_deref())?.print();
                }
                ShowCommand::ProxmoxStorage { host } => {
                    resolve_proxmox_storage_plan(&bundle, &host)?.print();
                }
            }
        }
        Command::Adapters { command } => {
            let registry = builtin_registry()?;
            match command {
                AdapterCommand::List => {
                    for adapter in registry.iter() {
                        let metadata = adapter.metadata();
                        println!("{}	{}	{}", metadata.id, metadata.name, metadata.description);
                    }
                }
            }
        }
        Command::Render { command } => {
            let bundle = ConfigLoader::new(&paths.config_dir).load(None)?;
            bundle.validate()?;
            let registry = builtin_registry()?;
            let plan = ResolvedPlan::new(&bundle, &paths);
            let context = AdapterContext::new(&plan);

            match command {
                RenderCommand::Bind => {
                    let rendered = registry.get("bind")?.render(&context)?;
                    println!("Rendered BIND configuration: {}", rendered.root.display());
                }
                RenderCommand::Nginx { host, check, nginx } => {
                    let context = match host.as_deref() {
                        Some(host) => context.for_host(host),
                        None => context,
                    };
                    let rendered = registry.get("nginx")?.render(&context)?;
                    if check {
                        validate_nginx_config(&rendered.root, &nginx)?;
                        println!("Validated Nginx configuration with {}", nginx.display());
                    }
                    println!("Rendered Nginx configuration: {}", rendered.root.display());
                }
                RenderCommand::Ssh { client } => {
                    let rendered = registry.get("ssh")?.render(&context.for_host(&client))?;
                    println!("Rendered SSH client config: {}", rendered.root.display());
                }
                RenderCommand::Docker { host } => {
                    let rendered = registry.get("docker")?.render(&context.for_host(&host))?;
                    println!("Rendered Docker networking: {}", rendered.root.display());
                }
                RenderCommand::Env { host } => {
                    let rendered = registry.get("env")?.render(&context.for_host(&host))?;
                    println!("Rendered host environment: {}", rendered.root.display());
                }
                RenderCommand::Netplan { host } => {
                    let rendered = registry.get("netplan")?.render(&context.for_host(&host))?;
                    println!(
                        "Rendered Netplan configuration: {}",
                        rendered.root.display()
                    );
                }
                RenderCommand::Proxmox { host } => {
                    let rendered = registry.get("proxmox")?.render(&context.for_host(&host))?;
                    println!(
                        "Rendered Proxmox configuration: {}",
                        rendered.root.display()
                    );
                }
                RenderCommand::SystemdMounts { host } => {
                    let rendered = registry
                        .get("systemd-mounts")?
                        .render(&context.for_host(&host))?;
                    println!(
                        "Rendered systemd network mounts: {}",
                        rendered.root.display()
                    );
                }
                RenderCommand::SynologyNfsPermissions { nas } => {
                    let rendered = registry
                        .get("synology-nfs-permissions")?
                        .render(&context.for_host(&nas))?;
                    println!(
                        "Rendered Synology NFS permission plan: {}",
                        rendered.root.display()
                    );
                }
                RenderCommand::ProxmoxGuests { host } => {
                    let rendered = registry
                        .get("proxmox-guests")?
                        .render(&context.for_host(&host))?;
                    println!("Rendered Proxmox guests: {}", rendered.root.display());
                }
                RenderCommand::ProxmoxSdn { host } => {
                    let rendered = registry
                        .get("proxmox-sdn")?
                        .render(&context.for_host(&host))?;
                    println!("Rendered Proxmox SDN: {}", rendered.root.display());
                }
                RenderCommand::ProxmoxStorage { host } => {
                    let rendered = registry
                        .get("proxmox-storage")?
                        .render(&context.for_host(&host))?;
                    println!("Rendered Proxmox storage: {}", rendered.root.display());
                }
                RenderCommand::All { host } => {
                    let bind = registry.get("bind")?.render(&context)?;
                    let env = registry.get("env")?.render(&context.for_host(&host))?;
                    println!("Rendered BIND configuration: {}", bind.root.display());
                    println!("Rendered host environment: {}", env.root.display());
                }
            }
        }
    }

    Ok(())
}
