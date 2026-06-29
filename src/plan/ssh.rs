use anyhow::{Context, Result, bail};
use std::net::IpAddr;

use crate::model::ConfigBundle;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSshPlan {
    pub location: String,
    pub client: String,
    pub targets: Vec<ResolvedSshTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSshTarget {
    pub alias: String,
    pub host_name: String,
    pub user: String,
    pub port: u16,
    pub identity_file: String,
    pub forward_agent: bool,
    pub source: String,
}

impl ResolvedSshPlan {
    pub fn print(&self) {
        println!(
            "SSH plan for client '{}' at '{}':",
            self.client, self.location
        );
        for target in &self.targets {
            println!(
                "  {} -> {}@{}:{} identity={} forward-agent={} source={}",
                target.alias,
                target.user,
                target.host_name,
                target.port,
                target.identity_file,
                target.forward_agent,
                target.source
            );
        }
    }
}

pub fn resolve_ssh_plan(bundle: &ConfigBundle, client_name: &str) -> Result<ResolvedSshPlan> {
    let client = bundle
        .ssh
        .clients
        .get(client_name)
        .with_context(|| format!("unknown SSH client profile '{client_name}'"))?;

    let mut targets = Vec::new();
    for (alias, target) in &bundle.ssh.targets {
        let kinds = [
            target.host.is_some(),
            target.guest.is_some(),
            target.service.is_some(),
        ]
        .into_iter()
        .filter(|value| *value)
        .count();
        if kinds != 1 {
            bail!("SSH target '{alias}' must set exactly one of host, guest, or service");
        }

        let identity = client.identities.get(&target.identity).with_context(|| {
            format!(
                "SSH target '{alias}' references unknown identity '{}' for client '{client_name}'",
                target.identity
            )
        })?;

        let (host_name, port, source) = if let Some(host_name) = &target.host {
            let inventory = bundle.inventory.hosts.get(host_name).with_context(|| {
                format!("SSH target '{alias}' references unknown host '{host_name}'")
            })?;
            if !inventory.enabled {
                continue;
            }
            let interface_name = target.interface.as_deref().unwrap_or("lan");
            let address = host_address(bundle, host_name, interface_name)?;
            (
                address,
                target.port,
                format!("host:{host_name}:{interface_name}"),
            )
        } else if let Some(guest_name) = &target.guest {
            let guest = bundle.guests.guests.get(guest_name).with_context(|| {
                format!("SSH target '{alias}' references unknown guest '{guest_name}'")
            })?;
            (
                guest.ipv4.to_string(),
                target.port,
                format!("guest:{guest_name}"),
            )
        } else {
            let service_name = target.service.as_deref().expect("validated service target");
            let service = bundle
                .services
                .services
                .get(service_name)
                .with_context(|| {
                    format!("SSH target '{alias}' references unknown service '{service_name}'")
                })?;
            if !service.enabled {
                continue;
            }
            let ssh = service.ssh.as_ref().with_context(|| {
                format!(
                    "SSH target '{alias}' references service '{service_name}' without SSH settings"
                )
            })?;
            let address = host_address(
                bundle,
                &service.host,
                target.interface.as_deref().unwrap_or("lan"),
            )?;
            (address, ssh.host_port, format!("service:{service_name}"))
        };

        targets.push(ResolvedSshTarget {
            alias: alias.clone(),
            host_name,
            user: target.user.clone(),
            port,
            identity_file: identity.file.clone(),
            forward_agent: target.forward_agent,
            source,
        });
    }

    targets.sort_by(|a, b| a.alias.cmp(&b.alias));

    Ok(ResolvedSshPlan {
        location: bundle.location.name.clone(),
        client: client_name.to_owned(),
        targets,
    })
}

fn host_address(bundle: &ConfigBundle, host: &str, interface: &str) -> Result<String> {
    if let Some(location_host) = bundle.location.hosts.get(host) {
        let interface = location_host
            .interfaces
            .get(interface)
            .with_context(|| format!("host '{host}' has no location interface '{interface}'"))?;
        let address = interface
            .ipv4
            .map(IpAddr::V4)
            .with_context(|| format!("host '{host}' interface has no IPv4 address"))?;
        return Ok(address.to_string());
    }

    if let Some(guest) = bundle.guests.guests.get(host) {
        return Ok(guest.ipv4.to_string());
    }

    bail!(
        "host '{host}' is neither attached to location '{}' nor defined as a guest",
        bundle.location.name
    )
}
