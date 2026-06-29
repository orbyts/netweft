use std::net::{Ipv4Addr, Ipv6Addr};

use anyhow::{Context, Result, bail};
use ipnet::{Ipv4Net, Ipv6Net};

use crate::model::{ConfigBundle, NetworkKind};
use crate::plan::address::network_ula;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDockerPlan {
    pub location: String,
    pub host: String,
    pub daemon: ResolvedDockerDaemon,
    pub networks: Vec<ResolvedDockerNetwork>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDockerDaemon {
    pub bridge_ipv4_cidr: Ipv4Net,
    pub bridge_ipv4_gateway: Ipv4Addr,
    pub bridge_ipv6_cidr: Ipv6Net,
    pub bridge_ipv6_gateway: Ipv6Addr,
    pub ipv4_pool_base: Ipv4Net,
    pub ipv4_pool_size: u8,
    pub ipv6_pool_base: Ipv6Net,
    pub ipv6_pool_size: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDockerNetwork {
    pub logical_name: String,
    pub docker_name: String,
    pub ipv4_cidr: Ipv4Net,
    pub ipv4_gateway: Ipv4Addr,
    pub ipv6_cidr: Option<Ipv6Net>,
    pub ipv6_gateway: Option<Ipv6Addr>,
    pub previous_ipv6_cidr: Option<Ipv6Net>,
}

impl ResolvedDockerPlan {
    pub fn print(&self) {
        println!("Docker network plan for '{}':", self.host);
        println!("  Location: {}", self.location);
        println!(
            "  docker0: {} gateway={} | {} gateway={}",
            self.daemon.bridge_ipv4_cidr,
            self.daemon.bridge_ipv4_gateway,
            self.daemon.bridge_ipv6_cidr,
            self.daemon.bridge_ipv6_gateway
        );
        println!(
            "  pools: {} /{} | {} /{}",
            self.daemon.ipv4_pool_base,
            self.daemon.ipv4_pool_size,
            self.daemon.ipv6_pool_base,
            self.daemon.ipv6_pool_size
        );
        for network in &self.networks {
            print!(
                "  {} ({}) {} gateway={}",
                network.logical_name, network.docker_name, network.ipv4_cidr, network.ipv4_gateway
            );
            if let (Some(cidr), Some(gateway)) = (network.ipv6_cidr, network.ipv6_gateway) {
                print!(" | {cidr} gateway={gateway}");
            }
            if let Some(previous) = network.previous_ipv6_cidr {
                print!(" migrate-from={previous}");
            }
            println!();
        }
    }
}

pub fn resolve_docker_plan(bundle: &ConfigBundle, host_name: &str) -> Result<ResolvedDockerPlan> {
    let host = bundle
        .inventory
        .hosts
        .get(host_name)
        .with_context(|| format!("unknown host '{host_name}'"))?;

    if !host.enabled {
        bail!("host '{host_name}' is disabled");
    }
    if !host.roles.iter().any(|role| role == "docker") {
        bail!("host '{host_name}' does not have the docker role");
    }

    let config = bundle
        .docker
        .hosts
        .get(host_name)
        .with_context(|| format!("host '{host_name}' has no docker.toml profile"))?;

    let mut networks = Vec::new();
    for (logical_name, network) in &bundle.networks.networks {
        if network.kind != NetworkKind::Docker || network.owner.as_deref() != Some(host_name) {
            continue;
        }

        let docker_name = network
            .docker_name
            .clone()
            .unwrap_or_else(|| logical_name.clone());
        let ipv4_cidr = network
            .ipv4_cidr
            .with_context(|| format!("Docker network '{logical_name}' requires ipv4_cidr"))?;
        let ipv4_gateway = network
            .ipv4_gateway
            .with_context(|| format!("Docker network '{logical_name}' requires ipv4_gateway"))?;

        let (ipv6_cidr, ipv6_gateway) = if network.ula_enabled {
            let cidr = network_ula(bundle, logical_name)?;
            (Some(cidr), Some(first_ipv6(cidr)?))
        } else {
            (None, None)
        };

        let previous_ipv6_cidr = config
            .network_migrations
            .get(&docker_name)
            .and_then(|migration| migration.previous_ipv6_cidr);

        networks.push(ResolvedDockerNetwork {
            logical_name: logical_name.clone(),
            docker_name,
            ipv4_cidr,
            ipv4_gateway,
            ipv6_cidr,
            ipv6_gateway,
            previous_ipv6_cidr,
        });
    }

    if networks.is_empty() {
        bail!("host '{host_name}' has no owned Docker networks");
    }

    Ok(ResolvedDockerPlan {
        location: bundle.location.name.clone(),
        host: host_name.to_owned(),
        daemon: ResolvedDockerDaemon {
            bridge_ipv4_cidr: config.bridge_ipv4_cidr,
            bridge_ipv4_gateway: first_ipv4(config.bridge_ipv4_cidr)?,
            bridge_ipv6_cidr: config.bridge_ipv6_cidr,
            bridge_ipv6_gateway: first_ipv6(config.bridge_ipv6_cidr)?,
            ipv4_pool_base: config.ipv4_pool_base,
            ipv4_pool_size: config.ipv4_pool_size,
            ipv6_pool_base: config.ipv6_pool_base,
            ipv6_pool_size: config.ipv6_pool_size,
        },
        networks,
    })
}

fn first_ipv4(network: Ipv4Net) -> Result<Ipv4Addr> {
    Ok(Ipv4Addr::from(
        u32::from(network.network())
            .checked_add(1)
            .context("IPv4 network has no gateway")?,
    ))
}

fn first_ipv6(network: Ipv6Net) -> Result<Ipv6Addr> {
    Ok(Ipv6Addr::from(
        u128::from(network.network())
            .checked_add(1)
            .context("IPv6 network has no gateway")?,
    ))
}
