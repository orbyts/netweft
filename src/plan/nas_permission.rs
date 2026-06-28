use std::net::Ipv4Addr;

use anyhow::{Context, Result};

use crate::model::{ConfigBundle, NasAccess, NasPermissionProvider, NasSquash};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedNasPermission {
    pub id: String,
    pub nas: String,
    pub nas_ipv4: Ipv4Addr,
    pub provider: NasPermissionProvider,
    pub share: String,
    pub client_host: String,
    pub client_ipv4: Ipv4Addr,
    pub access: NasAccess,
    pub squash: NasSquash,
    pub security: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolvedNasPermissionPlan {
    pub location: String,
    pub nas: Option<String>,
    pub permissions: Vec<ResolvedNasPermission>,
}

impl ResolvedNasPermissionPlan {
    pub fn print(&self) {
        println!("NAS permission plan for '{}':", self.location);
        for permission in &self.permissions {
            println!(
                "{}\tnas={}({})\tshare={}\tclient={}({})\taccess={}\tsquash={}\tsecurity={}",
                permission.id,
                permission.nas,
                permission.nas_ipv4,
                permission.share,
                permission.client_host,
                permission.client_ipv4,
                permission.access.as_str(),
                permission.squash.as_str(),
                permission.security,
            );
        }
    }
}

pub fn resolve_nas_permission_plan(
    bundle: &ConfigBundle,
    nas_filter: Option<&str>,
) -> Result<ResolvedNasPermissionPlan> {
    if let Some(nas) = nas_filter {
        bundle
            .inventory
            .hosts
            .get(nas)
            .with_context(|| format!("unknown NAS host '{nas}'"))?;
    }

    let mut permissions = Vec::new();

    for (id, permission) in &bundle.nas_permissions.permissions {
        if nas_filter.is_some_and(|nas| permission.nas != nas) {
            continue;
        }

        permissions.push(ResolvedNasPermission {
            id: id.clone(),
            nas: permission.nas.clone(),
            nas_ipv4: resolve_location_host_ipv4(bundle, &permission.nas, id, "NAS")?,
            provider: permission.provider,
            share: permission.share.clone(),
            client_host: permission.client_host.clone(),
            client_ipv4: resolve_client_ipv4(bundle, &permission.client_host, id)?,
            access: permission.access,
            squash: permission.squash,
            security: permission.security.clone(),
        });
    }

    permissions.sort_by(|left, right| left.id.cmp(&right.id));

    Ok(ResolvedNasPermissionPlan {
        location: bundle.location.name.clone(),
        nas: nas_filter.map(str::to_owned),
        permissions,
    })
}

fn resolve_client_ipv4(
    bundle: &ConfigBundle,
    client: &str,
    permission_id: &str,
) -> Result<Ipv4Addr> {
    if bundle.location.hosts.contains_key(client) {
        return resolve_location_host_ipv4(bundle, client, permission_id, "client");
    }

    let guest = bundle.guests.guests.get(client).with_context(|| {
        format!(
            "NAS permission '{permission_id}' client '{client}' is neither a host attached at \
             location '{}' nor a configured guest",
            bundle.location.name
        )
    })?;

    let parent = bundle.location.hosts.get(&guest.host).with_context(|| {
        format!(
            "NAS permission '{permission_id}' client guest '{client}' parent '{}' is not attached \
             at location '{}'",
            guest.host, bundle.location.name
        )
    })?;

    let interface = parent.interfaces.get(&guest.interface).with_context(|| {
        format!(
            "NAS permission '{permission_id}' client guest '{client}' references unknown location \
             interface '{}:{}'",
            guest.host, guest.interface
        )
    })?;

    let segment = bundle
        .location
        .segments
        .get(&interface.segment)
        .with_context(|| {
            format!(
                "NAS permission '{permission_id}' client guest '{client}' references unknown \
                 segment '{}'",
                interface.segment
            )
        })?;

    if !segment.ipv4_cidr.contains(&guest.ipv4) {
        anyhow::bail!(
            "NAS permission '{permission_id}' client guest '{client}' address {} is outside \
             segment '{}' ({})",
            guest.ipv4,
            interface.segment,
            segment.ipv4_cidr
        );
    }

    Ok(guest.ipv4)
}

fn resolve_location_host_ipv4(
    bundle: &ConfigBundle,
    host: &str,
    permission_id: &str,
    role: &str,
) -> Result<Ipv4Addr> {
    let location_host = bundle.location.hosts.get(host).with_context(|| {
        format!(
            "NAS permission '{permission_id}' {role} host '{host}' is not attached at location '{}'",
            bundle.location.name
        )
    })?;

    location_host
        .interfaces
        .get("lan")
        .and_then(|interface| interface.ipv4)
        .or_else(|| {
            location_host
                .interfaces
                .values()
                .find_map(|interface| interface.ipv4)
        })
        .with_context(|| {
            format!("NAS permission '{permission_id}' {role} host '{host}' has no IPv4 address")
        })
}
