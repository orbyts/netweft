use std::collections::BTreeSet;

use anyhow::{Result, bail};

use crate::model::{ConfigBundle, Ipv6Mode, RoutingMode, SCHEMA_VERSION, TailscaleStrategy};
use crate::plan::dns_access::derive_dns_access;
use crate::plan::proxy::resolve_proxy_plan;

#[derive(Debug, Default)]
pub struct ValidationReport {
    pub warnings: Vec<String>,
}

pub fn validate_bundle(bundle: &ConfigBundle) -> Result<ValidationReport> {
    validate_schema_versions(bundle)?;
    validate_location(bundle)?;
    validate_references(bundle)?;
    validate_addresses(bundle)?;
    resolve_proxy_plan(bundle)?;

    if bundle.dns.dns.recursion.enabled {
        derive_dns_access(bundle)?;
    }

    let mut report = ValidationReport::default();

    if bundle.location.ipv6.mode == Ipv6Mode::RouterAdvertised
        && bundle.location.ipv6.publish_public_aaaa
    {
        bail!(
            "location '{}' uses router-advertised IPv6 but requests durable public AAAA records",
            bundle.location.name
        );
    }

    if bundle.location.ipv6.mode == Ipv6Mode::RouterAdvertised
        && bundle.location.ipv6.prefix.is_some()
    {
        report.warnings.push(format!(
            "location '{}' has a dynamic router-advertised IPv6 prefix; Netweft will allow it for recursion but will not treat it as allocatable",
            bundle.location.name
        ));
    }

    if bundle.location.router.supports_vlans {
        let vlans = bundle
            .location
            .segments
            .values()
            .filter(|segment| segment.vlan_id.unwrap_or(0) != 0)
            .count();

        if vlans == 0 {
            report.warnings.push(format!(
                "router '{}' claims VLAN support but the location defines no VLAN segments",
                bundle.location.router.kind
            ));
        }
    }

    Ok(report)
}

fn validate_schema_versions(bundle: &ConfigBundle) -> Result<()> {
    let versions = [
        ("netweft.toml", bundle.settings.schema_version),
        ("inventory.toml", bundle.inventory.schema_version),
        ("networks.toml", bundle.networks.schema_version),
        ("services.toml", bundle.services.schema_version),
        ("dns.toml", bundle.dns.schema_version),
        ("allocations.toml", bundle.allocations.schema_version),
        ("location", bundle.location.schema_version),
    ];

    for (name, version) in versions {
        if version != SCHEMA_VERSION {
            bail!(
                "{name} uses schema version {version}, but this Netweft build supports version {SCHEMA_VERSION}"
            );
        }
    }

    Ok(())
}

fn validate_location(bundle: &ConfigBundle) -> Result<()> {
    if bundle.location.name != bundle.settings.active_location {
        // This is valid when the CLI overrides the active location, so it is
        // intentionally not rejected here.
    }

    match bundle.location.ipv6.mode {
        Ipv6Mode::Disabled => {
            if bundle.location.ipv6.prefix.is_some() {
                bail!("IPv6 mode is disabled but an IPv6 prefix is configured");
            }
        }
        Ipv6Mode::RouterAdvertised => {
            let prefix =
                bundle.location.ipv6.prefix.ok_or_else(|| {
                    anyhow::anyhow!("router-advertised IPv6 mode requires prefix")
                })?;

            if prefix.prefix_len() != 64 {
                bail!("router-advertised IPv6 prefix must be /64, got {prefix}");
            }
        }
        Ipv6Mode::Delegated => {
            let prefix = bundle
                .location
                .ipv6
                .prefix
                .ok_or_else(|| anyhow::anyhow!("delegated IPv6 mode requires prefix"))?;
            let subnet_length = bundle.location.ipv6.subnet_prefix_length.unwrap_or(64);

            if subnet_length != 64 {
                bail!("Netweft v0.1 only allocates /64 networks from delegated IPv6 prefixes");
            }

            if prefix.prefix_len() > subnet_length {
                bail!(
                    "delegated prefix {} is longer than the requested /{} subnet size",
                    prefix,
                    subnet_length
                );
            }
        }
    }

    Ok(())
}

fn validate_references(bundle: &ConfigBundle) -> Result<()> {
    for (name, host) in &bundle.inventory.hosts {
        if let Some(parent) = &host.parent
            && !bundle.inventory.hosts.contains_key(parent)
        {
            bail!("host '{name}' references unknown parent host '{parent}'");
        }
    }

    for (name, network) in &bundle.networks.networks {
        if let Some(owner) = &network.owner
            && !bundle.inventory.hosts.contains_key(owner)
        {
            bail!("network '{name}' references unknown owner host '{owner}'");
        }
    }

    for (name, service) in &bundle.services.services {
        if !bundle.inventory.hosts.contains_key(&service.host) {
            bail!(
                "service '{name}' references unknown host '{}'",
                service.host
            );
        }
        if !bundle.networks.networks.contains_key(&service.network) {
            bail!(
                "service '{name}' references unknown network '{}'",
                service.network
            );
        }

        if let Some(web) = &service.web
            && !bundle.services.services.contains_key(&web.proxy)
        {
            bail!(
                "service '{name}' references unknown proxy service '{}'",
                web.proxy
            );
        }
    }

    for (name, location_host) in &bundle.location.hosts {
        if !bundle.inventory.hosts.contains_key(name) {
            bail!("location references unknown host '{name}'");
        }

        for (interface_name, interface) in &location_host.interfaces {
            if !bundle.location.segments.contains_key(&interface.segment) {
                bail!(
                    "host '{name}' interface '{interface_name}' references unknown location segment '{}'",
                    interface.segment
                );
            }
        }
    }

    validate_network_routes(bundle)?;
    validate_tailscale(bundle)?;

    if !bundle
        .services
        .services
        .contains_key(&bundle.dns.dns.service)
    {
        bail!(
            "DNS configuration references unknown service '{}'",
            bundle.dns.dns.service
        );
    }

    for record in &bundle.dns.records {
        match record.kind {
            crate::model::DnsRecordKind::Host => {
                let target = record.target.as_ref().ok_or_else(|| {
                    anyhow::anyhow!("DNS host record '{}' requires target", record.name)
                })?;
                if !bundle.inventory.hosts.contains_key(target) {
                    bail!(
                        "DNS record '{}' references unknown host '{target}'",
                        record.name
                    );
                }
                if record.interface.is_none() {
                    bail!("DNS host record '{}' requires interface", record.name);
                }
            }
            crate::model::DnsRecordKind::Service | crate::model::DnsRecordKind::Proxy => {
                let target = record.target.as_ref().ok_or_else(|| {
                    anyhow::anyhow!("DNS service record '{}' requires target", record.name)
                })?;
                if !bundle.services.services.contains_key(target) {
                    bail!(
                        "DNS record '{}' references unknown service '{target}'",
                        record.name
                    );
                }
            }
            crate::model::DnsRecordKind::Cname => {
                if record.target.is_none() {
                    bail!("DNS CNAME record '{}' requires target", record.name);
                }
            }
            crate::model::DnsRecordKind::SegmentGateway => {
                let target = record.target.as_ref().ok_or_else(|| {
                    anyhow::anyhow!(
                        "DNS gateway record '{}' requires segment target",
                        record.name
                    )
                })?;
                if !bundle.location.segments.contains_key(target) {
                    bail!(
                        "DNS record '{}' references unknown segment '{target}'",
                        record.name
                    );
                }
            }
        }
    }

    for allocation_key in bundle
        .allocations
        .ula
        .as_ref()
        .into_iter()
        .flat_map(|ula| ula.networks.keys())
    {
        if !bundle
            .networks
            .networks
            .values()
            .any(|network| &network.allocation_key == allocation_key)
        {
            bail!("ULA allocation '{allocation_key}' does not match any network allocation_key");
        }
    }

    Ok(())
}

fn validate_network_routes(bundle: &ConfigBundle) -> Result<()> {
    for (name, network) in &bundle.networks.networks {
        let Some(routing) = &network.routing else {
            continue;
        };

        match routing.mode {
            RoutingMode::Direct => {
                if routing.via.is_some() {
                    bail!("network '{name}' uses direct routing but also defines 'via'");
                }
            }
            RoutingMode::ViaHost => {
                let via = routing.via.as_ref().ok_or_else(|| {
                    anyhow::anyhow!(
                        "network '{name}' uses via-host routing but does not define 'via'"
                    )
                })?;
                if !bundle.inventory.hosts.contains_key(via) {
                    bail!("network '{name}' references unknown gateway host '{via}'");
                }
            }
            RoutingMode::HostPrivate => {
                if routing.from.is_some() || routing.via.is_some() {
                    bail!("host-private network '{name}' must not define 'from' or 'via'");
                }
            }
        }

        if let Some(from) = &routing.from
            && !bundle.inventory.hosts.contains_key(from)
        {
            bail!("network '{name}' references unknown source host '{from}'");
        }
    }
    Ok(())
}

fn validate_tailscale(bundle: &ConfigBundle) -> Result<()> {
    let tailscale = &bundle.location.tailscale;

    if !tailscale.enabled {
        if tailscale.primary_router.is_some() || !tailscale.routers.is_empty() {
            bail!("Tailscale is disabled but router settings are present");
        }
        return Ok(());
    }

    let strategy = tailscale
        .strategy
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Tailscale is enabled but no strategy is configured"))?;

    match strategy {
        TailscaleStrategy::SubnetRouter => {
            let primary = tailscale
                .primary_router
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("subnet-router strategy requires primary_router"))?;
            if !tailscale.routers.contains_key(primary) {
                bail!("primary Tailscale router '{primary}' has no router configuration");
            }
        }
        TailscaleStrategy::HaSubnetRouter => {
            if tailscale.routers.len() < 2 {
                bail!("ha-subnet-router strategy requires at least two routers");
            }
        }
        TailscaleStrategy::DirectNodes => {}
    }

    for (router_name, router) in &tailscale.routers {
        if !bundle.inventory.hosts.contains_key(router_name) {
            bail!("Tailscale router '{router_name}' is not a known host");
        }

        for route_ref in &router.advertise {
            let Some((kind, name)) = route_ref.split_once(':') else {
                bail!("invalid advertise reference '{route_ref}'");
            };

            match kind {
                "segment" => {
                    if !bundle.location.segments.contains_key(name) {
                        bail!(
                            "Tailscale router '{router_name}' advertises unknown segment '{name}'"
                        );
                    }
                }
                "network" => {
                    let network = bundle.networks.networks.get(name).ok_or_else(|| {
                        anyhow::anyhow!(
                            "Tailscale router '{router_name}' advertises unknown network '{name}'"
                        )
                    })?;
                    let routing = network.routing.as_ref().ok_or_else(|| {
                        anyhow::anyhow!("advertised network '{name}' has no routing policy")
                    })?;

                    if routing.mode == RoutingMode::HostPrivate {
                        bail!("cannot advertise host-private network '{name}'");
                    }
                    if routing.from.as_deref() != Some(router_name) {
                        bail!("network '{name}' is not routed from '{router_name}'");
                    }
                }
                _ => bail!("unsupported advertise reference '{route_ref}'"),
            }
        }
    }

    Ok(())
}

fn validate_addresses(bundle: &ConfigBundle) -> Result<()> {
    for (name, segment) in &bundle.location.segments {
        if !segment.ipv4_cidr.contains(&segment.ipv4_gateway) {
            bail!(
                "segment '{name}' gateway {} is outside {}",
                segment.ipv4_gateway,
                segment.ipv4_cidr
            );
        }
    }

    for (host_name, host) in &bundle.location.hosts {
        for (interface_name, interface) in &host.interfaces {
            if let Some(ipv4) = interface.ipv4 {
                let segment = &bundle.location.segments[&interface.segment];
                if !segment.ipv4_cidr.contains(&ipv4) {
                    bail!(
                        "host '{host_name}' interface '{interface_name}' address {ipv4} is outside {}",
                        segment.ipv4_cidr
                    );
                }
            }
        }
    }

    for (name, network) in &bundle.networks.networks {
        if let (Some(cidr), Some(gateway)) = (network.ipv4_cidr, network.ipv4_gateway)
            && !cidr.contains(&gateway)
        {
            bail!("network '{name}' gateway {gateway} is outside {cidr}");
        }
    }

    let mut service_ipv4 = BTreeSet::new();
    let mut host_ports = BTreeSet::new();

    for (name, service) in &bundle.services.services {
        if let Some(address) = &service.address
            && let Some(ipv4) = address.ipv4
        {
            let network = &bundle.networks.networks[&service.network];
            let cidr = network.ipv4_cidr.ok_or_else(|| {
                anyhow::anyhow!(
                    "service '{name}' has IPv4 address {ipv4}, but network '{}' has no IPv4 CIDR",
                    service.network
                )
            })?;

            if !cidr.contains(&ipv4) {
                bail!(
                    "service '{name}' address {ipv4} is outside network '{}' ({cidr})",
                    service.network
                );
            }

            if !service_ipv4.insert(ipv4) {
                bail!("duplicate service IPv4 address detected: {ipv4}");
            }
        }

        for port in &service.ports {
            let key = (&service.host, port.host, format!("{:?}", port.protocol));
            if !host_ports.insert(key) {
                bail!(
                    "duplicate host port {}/{:?} on host '{}'",
                    port.host,
                    port.protocol,
                    service.host
                );
            }
        }

        if let Some(ssh) = &service.ssh {
            let key = (&service.host, ssh.host_port, "Tcp".to_owned());
            if !host_ports.insert(key) {
                bail!(
                    "duplicate host port {}/tcp on host '{}'",
                    ssh.host_port,
                    service.host
                );
            }
        }
    }

    Ok(())
}
