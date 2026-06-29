use std::net::Ipv4Addr;

use anyhow::{Context, Result, bail};

use crate::model::{
    ConfigBundle, HostOsIpv4Mode, HostOsIpv6Mode, HostOsNetworkProvider, HostOsNetworkRenderer,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedOsNetworkPlan {
    pub location: String,
    pub host: String,
    pub provider: HostOsNetworkProvider,
    pub renderer: HostOsNetworkRenderer,
    pub interface: String,
    pub ipv4_mode: HostOsIpv4Mode,
    pub expected_ipv4: Option<Ipv4Addr>,
    pub ipv6_mode: HostOsIpv6Mode,
}

impl ResolvedOsNetworkPlan {
    pub fn print(&self) {
        println!("OS network plan for '{}':", self.host);
        println!("  Location: {}", self.location);
        println!("  Provider: {}", self.provider.as_str());
        println!("  Renderer: {}", self.renderer.as_str());
        println!("  Interface: {}", self.interface);
        println!("  IPv4 mode: {}", self.ipv4_mode.as_str());
        if let Some(address) = self.expected_ipv4 {
            println!("  Expected IPv4: {address}");
        }
        println!("  IPv6 mode: {}", self.ipv6_mode.as_str());
    }
}

pub fn resolve_os_network_plan(
    bundle: &ConfigBundle,
    host_name: &str,
) -> Result<ResolvedOsNetworkPlan> {
    let host = bundle
        .inventory
        .hosts
        .get(host_name)
        .with_context(|| format!("unknown host '{host_name}'"))?;

    if !host.enabled {
        bail!("host '{host_name}' is disabled");
    }

    let network = host
        .os_network
        .as_ref()
        .with_context(|| format!("host '{host_name}' has no OS network profile"))?;

    if network.interface.trim().is_empty() {
        bail!("host '{host_name}' OS network interface is empty");
    }

    let guest = bundle.guests.guests.get(host_name).with_context(|| {
        format!("host '{host_name}' has an OS network profile but no matching guest declaration")
    })?;

    let parent = bundle.location.hosts.get(&guest.host).with_context(|| {
        format!(
            "host '{host_name}' guest parent '{}' is not attached at location '{}'",
            guest.host, bundle.location.name
        )
    })?;

    let location_interface = parent.interfaces.get(&guest.interface).with_context(|| {
        format!(
            "host '{host_name}' guest references unknown location interface '{}:{}'",
            guest.host, guest.interface
        )
    })?;

    let segment = bundle
        .location
        .segments
        .get(&location_interface.segment)
        .with_context(|| {
            format!(
                "host '{host_name}' guest interface references unknown segment '{}'",
                location_interface.segment
            )
        })?;

    if !segment.ipv4_cidr.contains(&guest.ipv4) {
        bail!(
            "host '{host_name}' expected IPv4 {} is outside segment '{}' ({})",
            guest.ipv4,
            location_interface.segment,
            segment.ipv4_cidr
        );
    }

    if network.ipv6_mode == HostOsIpv6Mode::RouterAdvertised
        && bundle.location.ipv6.mode == crate::model::Ipv6Mode::Disabled
    {
        bail!(
            "host '{host_name}' requests router-advertised IPv6, but location '{}' disables IPv6",
            bundle.location.name
        );
    }

    Ok(ResolvedOsNetworkPlan {
        location: bundle.location.name.clone(),
        host: host_name.to_owned(),
        provider: network.provider,
        renderer: network.renderer,
        interface: network.interface.clone(),
        ipv4_mode: network.ipv4_mode,
        expected_ipv4: match network.ipv4_mode {
            HostOsIpv4Mode::Dhcp => Some(guest.ipv4),
        },
        ipv6_mode: network.ipv6_mode,
    })
}
