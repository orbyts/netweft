use anyhow::Result;
use ipnet::{Ipv4Net, Ipv6Net};

use crate::model::{ConfigBundle, Ipv6Mode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDnsAccess {
    pub ipv4: Vec<Ipv4Net>,
    pub ipv6: Vec<Ipv6Net>,
}

pub fn derive_dns_access(bundle: &ConfigBundle) -> Result<ResolvedDnsAccess> {
    let policy = &bundle.dns.dns.recursion;

    if !policy.enabled {
        return Ok(ResolvedDnsAccess {
            ipv4: Vec::new(),
            ipv6: Vec::new(),
        });
    }

    let mut ipv4 = vec!["127.0.0.0/8".parse()?];
    let mut ipv6 = vec!["::1/128".parse()?];

    if policy.include_location_segments {
        for segment in bundle.location.segments.values() {
            if segment.dns_clients {
                push_unique_v4(&mut ipv4, segment.ipv4_cidr);
            }
        }

        if let Some(prefix) = bundle.location.ipv6.prefix {
            match bundle.location.ipv6.mode {
                Ipv6Mode::Disabled => {}
                Ipv6Mode::RouterAdvertised | Ipv6Mode::Delegated => {
                    push_unique_v6(&mut ipv6, prefix);
                }
            }
        }
    }

    if policy.include_docker_networks {
        let dns_service = &bundle.services.services[&bundle.dns.dns.service];

        for network in bundle.networks.networks.values() {
            let directly_reachable = network.routing.as_ref().is_some_and(|routing| {
                routing.mode == crate::model::RoutingMode::Direct
                    && routing.from.as_deref() == Some(dns_service.host.as_str())
            });

            if network.dns_clients
                && directly_reachable
                && let Some(cidr) = network.ipv4_cidr
            {
                push_unique_v4(&mut ipv4, cidr);
            }
        }
    }

    if policy.include_tailscale && bundle.location.tailscale.enabled {
        push_unique_v4(&mut ipv4, "100.64.0.0/10".parse()?);
        push_unique_v6(&mut ipv6, "fd7a:115c:a1e0::/48".parse()?);
    }

    if policy.include_ula
        && let Some(ula) = &bundle.allocations.ula
    {
        push_unique_v6(&mut ipv6, ula.prefix);
    }

    ipv4.sort_by_key(|network| (u32::from(network.network()), network.prefix_len()));
    ipv6.sort_by_key(|network| (u128::from(network.network()), network.prefix_len()));

    Ok(ResolvedDnsAccess { ipv4, ipv6 })
}

fn push_unique_v4(items: &mut Vec<Ipv4Net>, item: Ipv4Net) {
    if !items.contains(&item) {
        items.push(item);
    }
}

fn push_unique_v6(items: &mut Vec<Ipv6Net>, item: Ipv6Net) {
    if !items.contains(&item) {
        items.push(item);
    }
}
