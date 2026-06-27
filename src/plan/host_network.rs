use std::net::{Ipv4Addr, Ipv6Addr};

use anyhow::{Context, Result, bail};
use ipnet::Ipv4Net;

use crate::model::{ConfigBundle, HostNetworkProvider, InterfaceIpv6Mode, Ipv6Mode};
use crate::plan::dns::resolve_dns_plan;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolvedHostIpv6Mode {
    Disabled,
    Slaac,
    Static,
}

impl ResolvedHostIpv6Mode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Slaac => "slaac",
            Self::Static => "static",
        }
    }
}

#[derive(Debug)]
pub struct ResolvedHostNetworkPlan {
    pub location: String,
    pub host: String,
    pub fqdn: String,
    pub provider: HostNetworkProvider,
    pub management_interface: String,
    pub preserve_includes: bool,
    pub links: Vec<ResolvedHostLink>,
    pub bridges: Vec<ResolvedHostBridge>,
    pub dns_servers: Vec<Ipv4Addr>,
    pub search_domains: Vec<String>,
}

#[derive(Debug)]
pub struct ResolvedHostLink {
    pub name: String,
    pub kind: String,
}

#[derive(Debug)]
pub struct ResolvedHostBridge {
    pub name: String,
    pub ports: Vec<String>,
    pub location_interface: Option<String>,
    pub segment: Option<String>,
    pub ipv4: Option<Ipv4Net>,
    pub ipv4_gateway: Option<Ipv4Addr>,
    pub ipv6_mode: ResolvedHostIpv6Mode,
    pub public_ipv6_prefix: Option<String>,
    pub ula_address: Option<Ipv6Addr>,
    pub vlan_aware: bool,
    pub allowed_vlans: Option<String>,
    pub stp: bool,
    pub forward_delay: u32,
    pub comment: Option<String>,
}

impl ResolvedHostNetworkPlan {
    pub fn print(&self) {
        println!("Host network plan for '{}':", self.host);
        println!("  Location: {}", self.location);
        println!("  FQDN: {}", self.fqdn);
        println!("  Provider: {}", provider_name(self.provider));
        println!("  Management interface: {}", self.management_interface);
        println!(
            "  Preserve includes: {}",
            if self.preserve_includes { "yes" } else { "no" }
        );

        println!("  DNS:");
        for server in &self.dns_servers {
            println!("    server: {server}");
        }
        for domain in &self.search_domains {
            println!("    search: {domain}");
        }

        println!("  Links:");
        for link in &self.links {
            println!("    {} ({})", link.name, link.kind);
        }

        println!("  Bridges:");
        for bridge in &self.bridges {
            println!("    {}", bridge.name);
            println!(
                "      ports: {}",
                if bridge.ports.is_empty() {
                    "none".to_owned()
                } else {
                    bridge.ports.join(",")
                }
            );

            if let Some(interface) = &bridge.location_interface {
                println!("      location interface: {interface}");
            }

            if let Some(segment) = &bridge.segment {
                println!("      segment: {segment}");
            }

            if let Some(ipv4) = bridge.ipv4 {
                println!("      IPv4: {ipv4}");
            }

            if let Some(gateway) = bridge.ipv4_gateway {
                println!("      IPv4 gateway: {gateway}");
            }

            println!("      IPv6: {}", bridge.ipv6_mode.as_str());

            if let Some(prefix) = &bridge.public_ipv6_prefix {
                println!("      public IPv6 prefix: {prefix}");
            }

            if let Some(address) = bridge.ula_address {
                println!("      ULA: {address}");
            }

            println!(
                "      VLAN aware: {}",
                if bridge.vlan_aware { "yes" } else { "no" }
            );

            if let Some(vlans) = &bridge.allowed_vlans {
                println!("      allowed VLANs: {vlans}");
            }

            println!("      STP: {}", if bridge.stp { "on" } else { "off" });
            println!("      forwarding delay: {}", bridge.forward_delay);

            if let Some(comment) = &bridge.comment {
                println!("      comment: {comment}");
            }
        }
    }
}

pub fn resolve_host_network_plan(
    bundle: &ConfigBundle,
    host_name: &str,
) -> Result<ResolvedHostNetworkPlan> {
    let host = bundle
        .inventory
        .hosts
        .get(host_name)
        .with_context(|| format!("unknown host '{host_name}'"))?;

    if !host.enabled {
        bail!("host '{host_name}' is disabled");
    }

    let network = host
        .network
        .as_ref()
        .with_context(|| format!("host '{host_name}' has no host network profile"))?;

    let location_host = bundle.location.hosts.get(host_name).with_context(|| {
        format!(
            "host '{host_name}' is not attached to location '{}'",
            bundle.location.name
        )
    })?;

    let links = network
        .links
        .iter()
        .map(|link| ResolvedHostLink {
            name: link.name.clone(),
            kind: format!("{:?}", link.kind).to_ascii_lowercase(),
        })
        .collect();

    let mut management_bridge_count = 0usize;
    let mut bridges = Vec::new();

    for bridge in &network.bridges {
        let mut resolved = ResolvedHostBridge {
            name: bridge.name.clone(),
            ports: bridge.ports.clone(),
            location_interface: bridge.location_interface.clone(),
            segment: None,
            ipv4: None,
            ipv4_gateway: None,
            ipv6_mode: ResolvedHostIpv6Mode::Disabled,
            public_ipv6_prefix: None,
            ula_address: None,
            vlan_aware: bridge.vlan_aware,
            allowed_vlans: bridge.allowed_vlans.clone(),
            stp: bridge.stp,
            forward_delay: bridge.forward_delay,
            comment: bridge.comment.clone(),
        };

        if let Some(interface_name) = &bridge.location_interface {
            if interface_name == &network.management_interface {
                management_bridge_count += 1;
            }

            let interface = location_host
                .interfaces
                .get(interface_name)
                .with_context(|| {
                    format!(
                        "host '{host_name}' has no location interface \
                         '{interface_name}' at '{}'",
                        bundle.location.name
                    )
                })?;

            let segment = bundle
                .location
                .segments
                .get(&interface.segment)
                .with_context(|| {
                    format!(
                        "host '{host_name}' interface '{interface_name}' \
                         references unknown segment '{}'",
                        interface.segment
                    )
                })?;

            resolved.segment = Some(interface.segment.clone());
            resolved.ipv4_gateway = Some(segment.ipv4_gateway);

            if let Some(address) = interface.ipv4 {
                resolved.ipv4 = Some(
                    Ipv4Net::new(address, segment.ipv4_cidr.prefix_len()).with_context(|| {
                        format!(
                            "failed to combine {address} with segment \
                             prefix length {}",
                            segment.ipv4_cidr.prefix_len()
                        )
                    })?,
                );
            }

            resolved.ipv6_mode = match interface.ipv6_mode.as_ref() {
                Some(InterfaceIpv6Mode::Slaac) => ResolvedHostIpv6Mode::Slaac,
                Some(InterfaceIpv6Mode::Static) => ResolvedHostIpv6Mode::Static,
                Some(InterfaceIpv6Mode::Disabled) | None => ResolvedHostIpv6Mode::Disabled,
            };

            if resolved.ipv6_mode == ResolvedHostIpv6Mode::Slaac
                && bundle.location.ipv6.mode == Ipv6Mode::Disabled
            {
                bail!(
                    "host '{host_name}' interface '{interface_name}' \
                     requests SLAAC, but location '{}' disables IPv6",
                    bundle.location.name
                );
            }

            resolved.public_ipv6_prefix =
                bundle.location.ipv6.prefix.map(|prefix| prefix.to_string());

            if interface.ula_interface_id.is_some() {
                resolved.ula_address = Some(crate::plan::address::host_ula(
                    bundle,
                    host_name,
                    interface_name,
                )?);
            }
        }

        bridges.push(resolved);
    }

    if management_bridge_count == 0 {
        bail!(
            "host '{host_name}' has no bridge attached to management \
             interface '{}'",
            network.management_interface
        );
    }

    if management_bridge_count > 1 {
        bail!(
            "host '{host_name}' has multiple bridges attached to \
             management interface '{}'",
            network.management_interface
        );
    }

    let dns = resolve_dns_plan(bundle)?;

    Ok(ResolvedHostNetworkPlan {
        location: bundle.location.name.clone(),
        host: host_name.to_owned(),
        fqdn: format!("{host_name}.{}", bundle.inventory.domains.primary),
        provider: network.provider,
        management_interface: network.management_interface.clone(),
        preserve_includes: network.preserve_includes,
        links,
        bridges,
        dns_servers: vec![dns.ingress_ipv4],
        search_domains: vec![bundle.inventory.domains.primary.clone()],
    })
}

fn provider_name(provider: HostNetworkProvider) -> &'static str {
    match provider {
        HostNetworkProvider::ProxmoxIfupdown2 => "proxmox-ifupdown2",
    }
}
