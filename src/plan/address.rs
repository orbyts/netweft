use std::net::Ipv6Addr;

use anyhow::{Context, Result, bail};
use ipnet::Ipv6Net;

use crate::model::{ConfigBundle, Service};

pub fn ula_subnet(root: Ipv6Net, subnet_id: u16) -> Result<Ipv6Net> {
    if root.prefix_len() != 48 {
        bail!("ULA root must be /48, got {root}");
    }

    let address = Ipv6Addr::from(u128::from(root.network()) | ((subnet_id as u128) << 64));
    Ipv6Net::new(address, 64).context("failed to construct ULA /64")
}

pub fn interface_address(subnet: Ipv6Net, interface_id: &str) -> Result<Ipv6Addr> {
    if subnet.prefix_len() != 64 {
        bail!("interface addresses require a /64, got {subnet}");
    }

    let clean = interface_id.replace(':', "");
    let iid = u64::from_str_radix(&clean, 16)
        .with_context(|| format!("invalid hexadecimal IPv6 interface id '{interface_id}'"))?;

    Ok(Ipv6Addr::from(u128::from(subnet.network()) | iid as u128))
}

pub fn network_ula(bundle: &ConfigBundle, network: &str) -> Result<Ipv6Net> {
    let ula = bundle
        .allocations
        .ula
        .as_ref()
        .context("ULA is not allocated")?;
    let subnet_id = ula
        .networks
        .get(network)
        .with_context(|| format!("no ULA allocation for network '{network}'"))?;

    ula_subnet(ula.prefix, *subnet_id)
}

pub fn segment_ula(bundle: &ConfigBundle, segment: &str) -> Result<Ipv6Net> {
    let ula = bundle
        .allocations
        .ula
        .as_ref()
        .context("ULA is not allocated")?;
    let subnet_id = ula
        .segments
        .get(segment)
        .with_context(|| format!("no ULA allocation for segment '{segment}'"))?;

    ula_subnet(ula.prefix, *subnet_id)
}

pub fn service_ula(bundle: &ConfigBundle, service: &Service) -> Result<Ipv6Addr> {
    let address = service
        .address
        .as_ref()
        .context("service has no address block")?;
    let interface_id = address
        .ipv6_interface_id
        .as_deref()
        .context("service has no IPv6 interface id")?;

    interface_address(network_ula(bundle, &service.network)?, interface_id)
}

pub fn host_ula(bundle: &ConfigBundle, host: &str, interface: &str) -> Result<Ipv6Addr> {
    let iface = bundle
        .location
        .hosts
        .get(host)
        .with_context(|| format!("host '{host}' is not attached at this location"))?
        .interfaces
        .get(interface)
        .with_context(|| format!("host '{host}' has no interface '{interface}'"))?;
    let interface_id = iface.ula_interface_id.as_deref().with_context(|| {
        format!("host '{host}' interface '{interface}' has no ULA interface id")
    })?;

    interface_address(segment_ula(bundle, &iface.segment)?, interface_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_subnet_and_interface_address() {
        let root: Ipv6Net = "fd12:3456:789a::/48".parse().unwrap();
        let subnet = ula_subnet(root, 0x121).unwrap();

        assert_eq!(subnet.to_string(), "fd12:3456:789a:121::/64");
        assert_eq!(
            interface_address(subnet, "2929").unwrap().to_string(),
            "fd12:3456:789a:121::2929"
        );
    }
}
