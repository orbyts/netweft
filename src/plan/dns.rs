use std::collections::{BTreeMap, BTreeSet};
use std::net::{Ipv4Addr, Ipv6Addr};

use anyhow::{Context, Result};

use crate::model::{AddressFamily, AddressScope, ConfigBundle, DnsRecordKind, Ipv6Mode, Service};
use crate::plan::address::{host_ula, service_ula};
use crate::plan::dns_access::{ResolvedDnsAccess, derive_dns_access};

#[derive(Debug)]
pub struct ResolvedDnsPlan {
    pub location: String,
    pub provider: String,
    pub service: String,
    pub default_ttl: u32,
    pub negative_ttl: u32,
    pub soa_primary_nameserver: String,
    pub soa_responsible_mailbox: String,
    pub soa_refresh: u32,
    pub soa_retry: u32,
    pub soa_expire: u32,
    pub forwarders_ipv4: Vec<Ipv4Addr>,
    pub forwarders_ipv6: Vec<Ipv6Addr>,
    pub container_ipv4: Ipv4Addr,
    pub ingress_ipv4: Ipv4Addr,
    pub access: ResolvedDnsAccess,
    pub zones: Vec<ResolvedZone>,
    pub reverse_zones: Vec<ResolvedReverseZone>,
    pub warnings: Vec<String>,
}

#[derive(Debug)]
pub struct ResolvedZone {
    pub name: String,
    pub records: Vec<ResolvedRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResolvedRecord {
    A { name: String, address: Ipv4Addr },
    Aaaa { name: String, address: Ipv6Addr },
    Cname { name: String, target: String },
    Ns { name: String, target: String },
}

#[derive(Debug)]
pub struct ResolvedReverseZone {
    pub name: String,
    pub records: Vec<(String, String)>,
}

impl ResolvedDnsPlan {
    pub fn print(&self) {
        println!("DNS plan for '{}':", self.location);
        println!();
        println!("Server:");
        println!("  provider: {}", self.provider);
        println!("  service: {}", self.service);
        println!("  container IPv4: {}", self.container_ipv4);
        println!("  ingress IPv4: {}", self.ingress_ipv4);

        println!();
        println!("Recursion IPv4:");
        for network in &self.access.ipv4 {
            println!("  {network}");
        }
        println!("Recursion IPv6:");
        for network in &self.access.ipv6 {
            println!("  {network}");
        }

        for zone in &self.zones {
            println!();
            println!("Zone: {}", zone.name);
            for record in &zone.records {
                match record {
                    ResolvedRecord::A { name, address } => {
                        println!("  A      {name:<34} {address}");
                    }
                    ResolvedRecord::Aaaa { name, address } => {
                        println!("  AAAA   {name:<34} {address}");
                    }
                    ResolvedRecord::Cname { name, target } => {
                        println!("  CNAME  {name:<34} {target}");
                    }
                    ResolvedRecord::Ns { name, target } => {
                        println!("  NS     {name:<34} {target}");
                    }
                }
            }
        }

        for zone in &self.reverse_zones {
            println!();
            println!("Reverse zone: {}", zone.name);
            for (owner, target) in &zone.records {
                println!("  PTR    {owner:<34} {target}");
            }
        }

        if !self.warnings.is_empty() {
            println!();
            println!("Warnings:");
            for warning in &self.warnings {
                println!("  - {warning}");
            }
        }
    }
}

pub fn resolve_dns_plan(bundle: &ConfigBundle) -> Result<ResolvedDnsPlan> {
    let dns_service = bundle
        .services
        .services
        .get(&bundle.dns.dns.service)
        .context("DNS service is missing")?;

    let container_ipv4 = dns_service
        .address
        .as_ref()
        .and_then(|address| address.ipv4)
        .context("DNS service has no container IPv4 address")?;

    let ingress_ipv4 = resolve_service_ingress_ipv4(bundle, dns_service)?;
    let access = derive_dns_access(bundle)?;

    let mut zones: BTreeMap<String, BTreeSet<ResolvedRecord>> = BTreeMap::new();
    for zone in &bundle.dns.zones {
        zones
            .entry(zone.name.clone())
            .or_default()
            .insert(ResolvedRecord::Ns {
                name: fqdn(&zone.name),
                target: fqdn(&bundle.dns.dns.soa.primary_nameserver),
            });
    }

    add_a(
        &mut zones,
        &bundle.dns.dns.soa.primary_nameserver,
        ingress_ipv4,
    )?;

    let mut ptrs = Vec::new();
    let mut ptrs_v6 = Vec::new();

    for record in &bundle.dns.records {
        let publish_ipv4 = record.families.contains(&AddressFamily::Ipv4);
        let publish_ipv6 = record.families.contains(&AddressFamily::Ipv6);

        match record.kind {
            DnsRecordKind::Host => {
                let target = record
                    .target
                    .as_deref()
                    .context("host record target missing")?;
                let interface = record
                    .interface
                    .as_deref()
                    .context("host record interface missing")?;

                if publish_ipv4 {
                    let address = resolve_host_ipv4(bundle, target, interface)?;
                    add_a(&mut zones, &record.name, address)?;
                    if record.reverse {
                        ptrs.push((address, fqdn(&record.name)));
                    }
                }

                if publish_ipv6 {
                    let address = host_ula(bundle, target, interface)?;
                    add_aaaa(&mut zones, &record.name, address)?;
                    if record.reverse {
                        ptrs_v6.push((address, fqdn(&record.name)));
                    }
                }
            }
            DnsRecordKind::Service => {
                let target = record
                    .target
                    .as_deref()
                    .context("service record target missing")?;
                let service = &bundle.services.services[target];
                let scope = record
                    .address_scope
                    .as_ref()
                    .unwrap_or(&AddressScope::Container);

                if publish_ipv4 {
                    let address = match scope {
                        AddressScope::Container => service
                            .address
                            .as_ref()
                            .and_then(|value| value.ipv4)
                            .with_context(|| format!("service '{target}' has no IPv4 address"))?,
                        AddressScope::Ingress => resolve_service_ingress_ipv4(bundle, service)?,
                    };

                    add_a(&mut zones, &record.name, address)?;
                    if record.reverse {
                        ptrs.push((address, fqdn(&record.name)));
                    }
                }

                if publish_ipv6 {
                    let address = match scope {
                        AddressScope::Container => service_ula(bundle, service)?,
                        AddressScope::Ingress => {
                            let ingress = service
                                .ingress
                                .as_ref()
                                .context("service ingress missing")?;
                            host_ula(bundle, &service.host, &ingress.interface)?
                        }
                    };

                    add_aaaa(&mut zones, &record.name, address)?;
                    if record.reverse {
                        ptrs_v6.push((address, fqdn(&record.name)));
                    }
                }
            }
            DnsRecordKind::Proxy => {
                let target = record
                    .target
                    .as_deref()
                    .context("proxy record target missing")?;
                let proxy = &bundle.services.services[target];

                if publish_ipv4 {
                    add_a(
                        &mut zones,
                        &record.name,
                        resolve_service_ingress_ipv4(bundle, proxy)?,
                    )?;
                }

                if publish_ipv6 {
                    let ingress = proxy.ingress.as_ref().context("proxy ingress missing")?;
                    add_aaaa(
                        &mut zones,
                        &record.name,
                        host_ula(bundle, &proxy.host, &ingress.interface)?,
                    )?;
                }
            }
            DnsRecordKind::Cname => {
                let target = record.target.as_deref().context("CNAME target missing")?;
                add_record(
                    &mut zones,
                    &record.name,
                    ResolvedRecord::Cname {
                        name: fqdn(&record.name),
                        target: fqdn(target),
                    },
                )?;
            }
            DnsRecordKind::SegmentGateway => {
                let segment = record.target.as_deref().context("segment target missing")?;
                let address = bundle.location.segments[segment].ipv4_gateway;
                add_a(&mut zones, &record.name, address)?;
                if record.reverse {
                    ptrs.push((address, fqdn(&record.name)));
                }
            }
        }
    }

    // A web service explicitly configured for reverse-proxy access gets an
    // ingress record at its configured domain.
    for service in bundle.services.services.values() {
        if let Some(web) = &service.web
            && web.access == "reverse-proxy"
        {
            let proxy = &bundle.services.services[&web.proxy];
            add_a(
                &mut zones,
                &web.domain,
                resolve_service_ingress_ipv4(bundle, proxy)?,
            )?;
        }
    }

    let zones = zones
        .into_iter()
        .map(|(name, records)| ResolvedZone {
            name,
            records: records.into_iter().collect(),
        })
        .collect();

    let mut warnings = Vec::new();
    let dns_host = dns_service.host.as_str();

    for (name, network) in &bundle.networks.networks {
        if !network.dns_clients {
            continue;
        }

        let directly_reachable = network.routing.as_ref().is_some_and(|routing| {
            routing.mode == crate::model::RoutingMode::Direct
                && routing.from.as_deref() == Some(dns_host)
        });

        if !directly_reachable {
            let cidr = network
                .ipv4_cidr
                .map(|value| value.to_string())
                .unwrap_or_else(|| "no IPv4 CIDR".to_owned());

            warnings.push(format!(
                "network '{name}' ({cidr}) is excluded from DNS recursion because it is not directly reachable from DNS host '{dns_host}'"
            ));
        }
    }

    if bundle.location.ipv6.mode == Ipv6Mode::RouterAdvertised {
        warnings.push(format!(
            "location '{}' uses router-advertised IPv6; the prefix is accepted for recursion but no durable GUA AAAA records are emitted",
            bundle.location.name
        ));
    }

    Ok(ResolvedDnsPlan {
        location: bundle.location.name.clone(),
        provider: bundle.dns.dns.provider.clone(),
        service: bundle.dns.dns.service.clone(),
        default_ttl: bundle.dns.dns.default_ttl,
        negative_ttl: bundle.dns.dns.negative_ttl,
        soa_primary_nameserver: fqdn(&bundle.dns.dns.soa.primary_nameserver),
        soa_responsible_mailbox: fqdn(&bundle.dns.dns.soa.responsible_mailbox),
        soa_refresh: bundle.dns.dns.soa.refresh,
        soa_retry: bundle.dns.dns.soa.retry,
        soa_expire: bundle.dns.dns.soa.expire,
        forwarders_ipv4: bundle.dns.dns.forwarders.ipv4.clone(),
        forwarders_ipv6: bundle.dns.dns.forwarders.ipv6.clone(),
        container_ipv4,
        ingress_ipv4,
        access,
        zones,
        reverse_zones: build_reverse_zones(ptrs, ptrs_v6),
        warnings,
    })
}

fn resolve_service_ingress_ipv4(bundle: &ConfigBundle, service: &Service) -> Result<Ipv4Addr> {
    let ingress = service.ingress.as_ref().with_context(|| {
        format!(
            "service on host '{}' has no ingress definition",
            service.host
        )
    })?;

    resolve_host_ipv4(bundle, &service.host, &ingress.interface)
}

fn resolve_host_ipv4(bundle: &ConfigBundle, host: &str, interface: &str) -> Result<Ipv4Addr> {
    bundle
        .location
        .hosts
        .get(host)
        .with_context(|| {
            format!(
                "host '{host}' is not attached at location '{}'",
                bundle.location.name
            )
        })?
        .interfaces
        .get(interface)
        .with_context(|| format!("host '{host}' has no interface '{interface}'"))?
        .ipv4
        .with_context(|| format!("host '{host}' interface '{interface}' has no IPv4 address"))
}

fn add_a(
    zones: &mut BTreeMap<String, BTreeSet<ResolvedRecord>>,
    name: &str,
    address: Ipv4Addr,
) -> Result<()> {
    add_record(
        zones,
        name,
        ResolvedRecord::A {
            name: fqdn(name),
            address,
        },
    )
}

fn add_aaaa(
    zones: &mut BTreeMap<String, BTreeSet<ResolvedRecord>>,
    name: &str,
    address: Ipv6Addr,
) -> Result<()> {
    add_record(
        zones,
        name,
        ResolvedRecord::Aaaa {
            name: fqdn(name),
            address,
        },
    )
}

fn add_record(
    zones: &mut BTreeMap<String, BTreeSet<ResolvedRecord>>,
    name: &str,
    record: ResolvedRecord,
) -> Result<()> {
    let zone = matching_zone(zones, name)
        .with_context(|| format!("record '{name}' does not belong to a configured zone"))?;

    zones.get_mut(&zone).unwrap().insert(record);
    Ok(())
}

fn matching_zone(zones: &BTreeMap<String, BTreeSet<ResolvedRecord>>, name: &str) -> Option<String> {
    let clean = name.trim_end_matches('.');

    zones
        .keys()
        .filter(|zone| clean == zone.as_str() || clean.ends_with(&format!(".{zone}")))
        .max_by_key(|zone| zone.len())
        .cloned()
}

fn fqdn(value: &str) -> String {
    if value.ends_with('.') {
        value.to_owned()
    } else {
        format!("{value}.")
    }
}

fn build_reverse_zones(
    ptrs: Vec<(Ipv4Addr, String)>,
    ptrs_v6: Vec<(Ipv6Addr, String)>,
) -> Vec<ResolvedReverseZone> {
    let mut zones: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();

    for (address, target) in ptrs {
        let octets = address.octets();
        let zone = format!("{}.{}.{}.in-addr.arpa", octets[2], octets[1], octets[0]);

        zones
            .entry(zone)
            .or_default()
            .push((octets[3].to_string(), target));
    }

    for (address, target) in ptrs_v6 {
        let full = format!("{:032x}", u128::from(address));
        let zone_part: String = full[..16]
            .chars()
            .rev()
            .map(|character| format!("{character}."))
            .collect();
        let owner: String = full[16..]
            .chars()
            .rev()
            .map(|character| format!("{character}."))
            .collect::<String>()
            .trim_end_matches('.')
            .to_owned();

        zones
            .entry(format!("{zone_part}ip6.arpa"))
            .or_default()
            .push((owner, target));
    }

    zones
        .into_iter()
        .map(|(name, mut records)| {
            records.sort();
            ResolvedReverseZone { name, records }
        })
        .collect()
}
