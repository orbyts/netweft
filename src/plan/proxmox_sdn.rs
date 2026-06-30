use std::net::Ipv4Addr;

use anyhow::{Context, Result, bail};
use ipnet::Ipv4Net;

use crate::model::{ConfigBundle, ProxmoxSdnDhcp, ProxmoxSdnIpam, ProxmoxSdnZoneKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedProxmoxSdnPlan {
    pub location: String,
    pub host: String,
    pub zones: Vec<ResolvedProxmoxSdnZone>,
    pub vnets: Vec<ResolvedProxmoxSdnVnet>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedProxmoxSdnZone {
    pub name: String,
    pub kind: ProxmoxSdnZoneKind,
    pub ipam: ProxmoxSdnIpam,
    pub dhcp: ProxmoxSdnDhcp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedProxmoxSdnVnet {
    pub name: String,
    pub zone: String,
    pub alias: Option<String>,
    pub subnets: Vec<ResolvedProxmoxSdnSubnet>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedProxmoxSdnSubnet {
    pub id: String,
    pub cidr: Ipv4Net,
    pub gateway: Ipv4Addr,
    pub dhcp_start: Ipv4Addr,
    pub dhcp_end: Ipv4Addr,
    pub snat: bool,
}

impl ResolvedProxmoxSdnPlan {
    pub fn print(&self) {
        println!("Proxmox SDN plan for '{}':", self.host);
        println!("  Location: {}", self.location);

        println!("  Zones:");
        for zone in &self.zones {
            println!(
                "    {} kind={} ipam={} dhcp={}",
                zone.name,
                zone.kind.as_str(),
                zone.ipam.as_str(),
                zone.dhcp.as_str(),
            );
        }

        println!("  VNets:");
        for vnet in &self.vnets {
            println!("    {} zone={}", vnet.name, vnet.zone);
            for subnet in &vnet.subnets {
                println!(
                    "      {} gateway={} dhcp={}-{} snat={}",
                    subnet.cidr, subnet.gateway, subnet.dhcp_start, subnet.dhcp_end, subnet.snat,
                );
            }
        }
    }
}

pub fn resolve_proxmox_sdn_plan(
    bundle: &ConfigBundle,
    host_name: &str,
) -> Result<ResolvedProxmoxSdnPlan> {
    let host = bundle
        .inventory
        .hosts
        .get(host_name)
        .with_context(|| format!("unknown host '{host_name}'"))?;

    if !host.enabled {
        bail!("host '{host_name}' is disabled");
    }

    let mut zones = Vec::new();
    for (name, zone) in &bundle.proxmox_sdn.zones {
        if zone.host != host_name {
            continue;
        }

        zones.push(ResolvedProxmoxSdnZone {
            name: name.clone(),
            kind: zone.kind,
            ipam: zone.ipam,
            dhcp: zone.dhcp,
        });
    }

    if zones.is_empty() {
        bail!("host '{host_name}' has no Proxmox SDN zones");
    }

    let zone_names: std::collections::BTreeSet<_> =
        zones.iter().map(|zone| zone.name.as_str()).collect();

    let mut vnets = Vec::new();
    for (name, vnet) in &bundle.proxmox_sdn.vnets {
        if !zone_names.contains(vnet.zone.as_str()) {
            continue;
        }

        let mut subnets = Vec::new();
        for subnet in &vnet.subnets {
            if !subnet.cidr.contains(&subnet.gateway) {
                bail!(
                    "Proxmox SDN VNet '{name}' gateway {} is outside {}",
                    subnet.gateway,
                    subnet.cidr
                );
            }

            if !subnet.cidr.contains(&subnet.dhcp_start) || !subnet.cidr.contains(&subnet.dhcp_end)
            {
                bail!(
                    "Proxmox SDN VNet '{name}' DHCP range {}-{} is outside {}",
                    subnet.dhcp_start,
                    subnet.dhcp_end,
                    subnet.cidr
                );
            }

            if subnet.dhcp_start > subnet.dhcp_end {
                bail!(
                    "Proxmox SDN VNet '{name}' DHCP start {} is after end {}",
                    subnet.dhcp_start,
                    subnet.dhcp_end
                );
            }

            if subnet.gateway >= subnet.dhcp_start && subnet.gateway <= subnet.dhcp_end {
                bail!(
                    "Proxmox SDN VNet '{name}' gateway {} overlaps DHCP range",
                    subnet.gateway
                );
            }

            subnets.push(ResolvedProxmoxSdnSubnet {
                id: subnet_id(&vnet.zone, subnet.cidr),
                cidr: subnet.cidr,
                gateway: subnet.gateway,
                dhcp_start: subnet.dhcp_start,
                dhcp_end: subnet.dhcp_end,
                snat: subnet.snat,
            });
        }

        vnets.push(ResolvedProxmoxSdnVnet {
            name: name.clone(),
            zone: vnet.zone.clone(),
            alias: vnet.alias.clone(),
            subnets,
        });
    }

    if vnets.is_empty() {
        bail!("host '{host_name}' has no Proxmox SDN VNets");
    }

    Ok(ResolvedProxmoxSdnPlan {
        location: bundle.location.name.clone(),
        host: host_name.to_owned(),
        zones,
        vnets,
    })
}

fn subnet_id(zone: &str, cidr: Ipv4Net) -> String {
    format!("{}-{}-{}", zone, cidr.network(), cidr.prefix_len())
}
