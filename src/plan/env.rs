use std::collections::BTreeMap;
use std::net::Ipv6Addr;

use anyhow::{Context, Result};

use crate::model::ConfigBundle;
use crate::paths::NetweftPaths;
use crate::plan::address::{host_ula, network_ula, service_ula};

#[derive(Debug)]
pub struct ResolvedEnvPlan {
    pub location: String,
    pub host: String,
    pub variables: BTreeMap<String, String>,
}

impl ResolvedEnvPlan {
    pub fn print(&self) {
        println!(
            "Environment plan for host '{}' at '{}':",
            self.host, self.location
        );

        for (key, value) in &self.variables {
            println!("  {key}={value}");
        }
    }
}

pub fn resolve_env_plan(
    bundle: &ConfigBundle,
    paths: &NetweftPaths,
    host: &str,
) -> Result<ResolvedEnvPlan> {
    let inventory_host = bundle
        .inventory
        .hosts
        .get(host)
        .with_context(|| format!("unknown host '{host}'"))?;
    let location_host = bundle.location.hosts.get(host).with_context(|| {
        format!(
            "host '{host}' is not attached to location '{}'",
            bundle.location.name
        )
    })?;

    let mut variables = BTreeMap::new();
    let location_root = paths.generated_dir.join(&bundle.location.name);

    variables.insert("NETWEFT_LOCATION".into(), bundle.location.name.clone());
    variables.insert(
        "NETWEFT_GENERATED_ROOT".into(),
        location_root.display().to_string(),
    );
    variables.insert(
        "NETWEFT_BIND_CONFIG_DIR".into(),
        location_root.join("bind").display().to_string(),
    );

    let host_key = env_key(host);

    if let Some(runtime_root) = &inventory_host.runtime_root {
        variables.insert(format!("{host_key}_RUNTIME"), runtime_root.clone());
        variables.insert(
            "TAILSCALE_STATE_DIR".into(),
            format!("{runtime_root}/tailscale/state"),
        );
    }

    variables.insert("TAILSCALE_HOSTNAME".into(), host.to_owned());

    for (interface_name, interface) in &location_host.interfaces {
        let key = format!("{}_{}", host_key, env_key(interface_name));

        if let Some(address) = interface.ipv4 {
            variables.insert(format!("{key}_IPV4"), address.to_string());

            if interface_name == "lan" {
                variables.insert(format!("{host_key}_LAN_IPV4"), address.to_string());
            }
        }

        if interface.ula_interface_id.is_some() {
            let address = host_ula(bundle, host, interface_name)?;
            variables.insert(format!("{key}_IPV6"), address.to_string());

            if interface_name == "lan" {
                variables.insert(format!("{host_key}_LAN_IPV6"), address.to_string());
            }
        }
    }

    for (name, network) in &bundle.networks.networks {
        if network.owner.as_deref() != Some(host) {
            continue;
        }

        let key = env_key(network.docker_name.as_deref().unwrap_or(name));

        if let Some(cidr) = network.ipv4_cidr {
            variables.insert(format!("{key}_IPV4_SUBNET"), cidr.to_string());
        }

        if let Some(gateway) = network.ipv4_gateway {
            variables.insert(format!("{key}_IPV4_GATEWAY"), gateway.to_string());
        }

        if network.ula_enabled {
            let subnet = network_ula(bundle, name)?;
            variables.insert(format!("{key}_IPV6_SUBNET"), subnet.to_string());
            variables.insert(
                format!("{key}_IPV6_GATEWAY"),
                Ipv6Addr::from(u128::from(subnet.network()) + 1).to_string(),
            );
        }
    }

    for (name, service) in &bundle.services.services {
        if !service.enabled || service.host != host {
            continue;
        }

        let key = env_key(name);

        if let Some(address) = service.address.as_ref().and_then(|value| value.ipv4) {
            variables.insert(format!("{key}_IPV4"), address.to_string());
        }

        if service
            .address
            .as_ref()
            .and_then(|value| value.ipv6_interface_id.as_ref())
            .is_some()
        {
            variables.insert(
                format!("{key}_IPV6"),
                service_ula(bundle, service)?.to_string(),
            );
        }
    }

    if let Some(router) = bundle.location.tailscale.routers.get(host) {
        let mut routes = Vec::new();

        for advertised in &router.advertise {
            if let Some(segment) = advertised.strip_prefix("segment:") {
                routes.push(bundle.location.segments[segment].ipv4_cidr.to_string());

                for (interface_name, interface) in &location_host.interfaces {
                    if interface.segment == segment && interface.ula_interface_id.is_some() {
                        routes.push(format!("{}/128", host_ula(bundle, host, interface_name)?));
                    }
                }
            } else if let Some(network) = advertised.strip_prefix("network:") {
                let logical = &bundle.networks.networks[network];

                if let Some(cidr) = logical.ipv4_cidr {
                    routes.push(cidr.to_string());
                }

                if logical.ula_enabled {
                    routes.push(network_ula(bundle, network)?.to_string());
                }
            }
        }

        variables.insert("TS_ADVERTISE_ROUTES".into(), routes.join(","));
    }

    Ok(ResolvedEnvPlan {
        location: bundle.location.name.clone(),
        host: host.to_owned(),
        variables,
    })
}

fn env_key(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect()
}
