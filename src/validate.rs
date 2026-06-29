use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Result, bail};

use crate::model::{
    ConfigBundle, GuestKind, Ipv6Mode, RoutingMode, SCHEMA_VERSION, TailscaleStrategy,
};
use crate::plan::dns_access::derive_dns_access;
use crate::plan::os_network::resolve_os_network_plan;
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
    validate_host_networks(bundle)?;
    validate_os_networks(bundle)?;
    validate_guest_and_mount_references(bundle)?;
    validate_storage_and_nas_references(bundle)?;
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

fn validate_os_networks(bundle: &ConfigBundle) -> Result<()> {
    for (host_name, host) in &bundle.inventory.hosts {
        if host.os_network.is_some() {
            resolve_os_network_plan(bundle, host_name)?;
        }
    }
    Ok(())
}

fn validate_guest_and_mount_references(bundle: &ConfigBundle) -> Result<()> {
    let mut vmids = BTreeSet::new();
    let mut macs = BTreeSet::new();
    let mut addresses = BTreeSet::new();
    let mut pci = BTreeMap::<String, String>::new();

    for (name, guest) in &bundle.guests.guests {
        if !bundle.inventory.hosts.contains_key(&guest.host) {
            bail!(
                "guest '{name}' references unknown parent host '{}'",
                guest.host
            );
        }
        if !vmids.insert((guest.host.clone(), guest.vmid)) {
            bail!(
                "guest '{name}' duplicates VMID {} on host '{}'",
                guest.vmid,
                guest.host
            );
        }
        let mac = guest.mac.to_ascii_uppercase();
        if !macs.insert(mac.clone()) {
            bail!("guest '{name}' duplicates MAC '{mac}'");
        }
        if !addresses.insert(guest.ipv4) {
            bail!("guest '{name}' duplicates IPv4 {}", guest.ipv4);
        }
        if guest.kind != GuestKind::Vm
            && (!guest.pci_devices.is_empty() || !guest.virtiofs.is_empty())
        {
            bail!("guest '{name}' is not a VM but declares VM-only attachments");
        }

        let mut pci_slots = BTreeSet::new();
        for attachment in &guest.pci_devices {
            if !pci_slots.insert(attachment.slot) {
                bail!(
                    "guest '{name}' duplicates PCI attachment slot {}",
                    attachment.slot
                );
            }
            if attachment.device.trim().is_empty() {
                bail!(
                    "guest '{name}' PCI attachment slot {} has an empty device",
                    attachment.slot
                );
            }
            if let Some(owner) = pci.insert(attachment.device.clone(), name.clone()) {
                bail!(
                    "guests '{owner}' and '{name}' both claim PCI device '{}'",
                    attachment.device
                );
            }
        }

        let mut virtiofs_slots = BTreeSet::new();
        for attachment in &guest.virtiofs {
            if !virtiofs_slots.insert(attachment.slot) {
                bail!(
                    "guest '{name}' duplicates VirtioFS attachment slot {}",
                    attachment.slot
                );
            }
            if attachment.directory.trim().is_empty() {
                bail!(
                    "guest '{name}' VirtioFS attachment slot {} has an empty directory",
                    attachment.slot
                );
            }
        }
    }

    for (id, mount) in &bundle.mounts.mounts {
        if !bundle.inventory.hosts.contains_key(&mount.host) {
            bail!(
                "network mount '{id}' references unknown consumer host '{}'",
                mount.host
            );
        }
        if !bundle.inventory.hosts.contains_key(&mount.server_host) {
            bail!(
                "network mount '{id}' references unknown server host '{}'",
                mount.server_host
            );
        }
        if !mount.mount_path.starts_with('/') {
            bail!(
                "network mount '{id}' path '{}' must be absolute",
                mount.mount_path
            );
        }
    }
    Ok(())
}

fn validate_storage_and_nas_references(bundle: &ConfigBundle) -> Result<()> {
    let mut storage_ids = BTreeSet::new();

    for (id, permission) in &bundle.nas_permissions.permissions {
        if !bundle.inventory.hosts.contains_key(&permission.nas) {
            bail!(
                "NAS permission '{id}' references unknown NAS host '{}'",
                permission.nas
            );
        }
        if !bundle.inventory.hosts.contains_key(&permission.client_host) {
            bail!(
                "NAS permission '{id}' references unknown client host '{}'",
                permission.client_host
            );
        }
        if permission.share.trim().is_empty() {
            bail!("NAS permission '{id}' has an empty share name");
        }
        if permission.security.trim().is_empty() {
            bail!("NAS permission '{id}' has empty security mode");
        }
    }

    for (id, storage) in &bundle.proxmox_storages.storages {
        if !bundle.inventory.hosts.contains_key(&storage.host) {
            bail!(
                "Proxmox storage '{id}' references unknown host '{}'",
                storage.host
            );
        }
        if !bundle.inventory.hosts.contains_key(&storage.server_host) {
            bail!(
                "Proxmox storage '{id}' references unknown server host '{}'",
                storage.server_host
            );
        }
        if !storage.mount_path.starts_with('/') {
            bail!(
                "Proxmox storage '{id}' path '{}' must be absolute",
                storage.mount_path
            );
        }
        if storage.export.trim().is_empty() || !storage.export.starts_with('/') {
            bail!(
                "Proxmox storage '{id}' export '{}' must be an absolute export path",
                storage.export
            );
        }
        if storage.content.is_empty() {
            bail!("Proxmox storage '{id}' must declare at least one content type");
        }
        if !storage_ids.insert((storage.host.clone(), storage.storage_id.clone())) {
            bail!(
                "Proxmox storage '{id}' duplicates storage ID '{}' on host '{}'",
                storage.storage_id,
                storage.host
            );
        }
    }
    Ok(())
}

fn validate_schema_versions(bundle: &ConfigBundle) -> Result<()> {
    let versions = [
        ("netweft.toml", bundle.settings.schema_version),
        ("inventory.toml", bundle.inventory.schema_version),
        ("networks.toml", bundle.networks.schema_version),
        ("services.toml", bundle.services.schema_version),
        ("guests.toml", bundle.guests.schema_version),
        ("mounts.toml", bundle.mounts.schema_version),
        (
            "nas-permissions.toml",
            bundle.nas_permissions.schema_version,
        ),
        (
            "proxmox-storages.toml",
            bundle.proxmox_storages.schema_version,
        ),
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

fn validate_host_networks(bundle: &ConfigBundle) -> Result<()> {
    let mut static_ipv4_owners = BTreeMap::new();

    for (host_name, host) in &bundle.inventory.hosts {
        let Some(network) = &host.network else {
            continue;
        };

        if !host.enabled {
            continue;
        }

        let location_host = bundle.location.hosts.get(host_name).ok_or_else(|| {
            anyhow::anyhow!(
                "host '{host_name}' defines a network profile but is not attached to location '{}'",
                bundle.location.name
            )
        })?;

        let mut link_names = BTreeSet::new();

        for link in &network.links {
            if link.name.trim().is_empty() {
                bail!("host '{host_name}' defines an empty network link name");
            }

            if !link_names.insert(link.name.as_str()) {
                bail!(
                    "host '{host_name}' defines duplicate network link '{}'",
                    link.name
                );
            }
        }

        let mut bridge_names = BTreeSet::new();
        let mut claimed_links: BTreeMap<&str, &str> = BTreeMap::new();
        let mut management_bridge_count = 0usize;

        for bridge in &network.bridges {
            if bridge.name.trim().is_empty() {
                bail!("host '{host_name}' defines an empty bridge name");
            }

            if !bridge_names.insert(bridge.name.as_str()) {
                bail!(
                    "host '{host_name}' defines duplicate bridge '{}'",
                    bridge.name
                );
            }

            if bridge.allowed_vlans.is_some() && !bridge.vlan_aware {
                bail!(
                    "host '{host_name}' bridge '{}' defines allowed_vlans but is not VLAN-aware",
                    bridge.name
                );
            }

            if let Some(vlans) = &bridge.allowed_vlans {
                validate_vlan_expression(host_name, &bridge.name, vlans)?;
            }

            for port in &bridge.ports {
                if !link_names.contains(port.as_str()) {
                    bail!(
                        "host '{host_name}' bridge '{}' references unknown link '{port}'",
                        bridge.name
                    );
                }

                if let Some(existing_bridge) =
                    claimed_links.insert(port.as_str(), bridge.name.as_str())
                {
                    bail!(
                        "host '{host_name}' link '{port}' is assigned to both bridge '{existing_bridge}' and bridge '{}'",
                        bridge.name
                    );
                }
            }

            let Some(interface_name) = &bridge.location_interface else {
                continue;
            };

            if interface_name == &network.management_interface {
                management_bridge_count += 1;
            }

            let interface = location_host
                .interfaces
                .get(interface_name)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "host '{host_name}' bridge '{}' references missing location interface '{interface_name}'",
                        bridge.name
                    )
                })?;

            let segment = bundle
                .location
                .segments
                .get(&interface.segment)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "host '{host_name}' interface '{interface_name}' references unknown segment '{}'",
                        interface.segment
                    )
                })?;

            if let Some(ipv4) = interface.ipv4 {
                if ipv4 == segment.ipv4_gateway {
                    bail!(
                        "host '{host_name}' interface '{interface_name}' address {ipv4} conflicts with segment '{}' gateway",
                        interface.segment
                    );
                }

                if let Some((other_host, other_interface)) =
                    static_ipv4_owners.insert(ipv4, (host_name.as_str(), interface_name.as_str()))
                {
                    bail!(
                        "duplicate host IPv4 address {ipv4}: '{other_host}.{other_interface}' and '{host_name}.{interface_name}'"
                    );
                }
            }

            if matches!(
                interface.ipv6_mode,
                Some(crate::model::InterfaceIpv6Mode::Slaac)
            ) && bundle.location.ipv6.mode == Ipv6Mode::Disabled
            {
                bail!(
                    "host '{host_name}' interface '{interface_name}' requests SLAAC, but location '{}' disables IPv6",
                    bundle.location.name
                );
            }
        }

        if management_bridge_count == 0 {
            bail!(
                "host '{host_name}' has no bridge attached to management interface '{}'",
                network.management_interface
            );
        }

        if management_bridge_count > 1 {
            bail!(
                "host '{host_name}' has multiple bridges attached to management interface '{}'",
                network.management_interface
            );
        }

        if !location_host
            .interfaces
            .contains_key(&network.management_interface)
        {
            bail!(
                "host '{host_name}' management interface '{}' is not defined at location '{}'",
                network.management_interface,
                bundle.location.name
            );
        }
    }

    Ok(())
}

fn validate_vlan_expression(host_name: &str, bridge_name: &str, expression: &str) -> Result<()> {
    if expression.trim().is_empty() {
        bail!("host '{host_name}' bridge '{bridge_name}' has an empty allowed_vlans value");
    }

    for item in expression.split(',') {
        let item = item.trim();

        if item.is_empty() {
            bail!(
                "host '{host_name}' bridge '{bridge_name}' has an invalid VLAN expression '{expression}'"
            );
        }

        if let Some((start, end)) = item.split_once('-') {
            let start: u16 = start.parse().map_err(|_| {
                anyhow::anyhow!(
                    "host '{host_name}' bridge '{bridge_name}' has invalid VLAN value '{item}'"
                )
            })?;

            let end: u16 = end.parse().map_err(|_| {
                anyhow::anyhow!(
                    "host '{host_name}' bridge '{bridge_name}' has invalid VLAN value '{item}'"
                )
            })?;

            validate_vlan_id(host_name, bridge_name, start)?;
            validate_vlan_id(host_name, bridge_name, end)?;

            if start > end {
                bail!(
                    "host '{host_name}' bridge '{bridge_name}' has descending VLAN range '{item}'"
                );
            }
        } else {
            let vlan: u16 = item.parse().map_err(|_| {
                anyhow::anyhow!(
                    "host '{host_name}' bridge '{bridge_name}' has invalid VLAN value '{item}'"
                )
            })?;

            validate_vlan_id(host_name, bridge_name, vlan)?;
        }
    }

    Ok(())
}

fn validate_vlan_id(host_name: &str, bridge_name: &str, vlan: u16) -> Result<()> {
    if !(1..=4094).contains(&vlan) {
        bail!("host '{host_name}' bridge '{bridge_name}' VLAN {vlan} is outside 1-4094");
    }

    Ok(())
}
