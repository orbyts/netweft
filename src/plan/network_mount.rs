use std::net::Ipv4Addr;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::model::{ConfigBundle, NetworkMountProvider};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedNetworkMount {
    pub id: String,
    pub host: String,
    pub provider: NetworkMountProvider,
    pub server_host: String,
    pub server_ipv4: Ipv4Addr,
    pub export: String,
    pub mount_path: PathBuf,
    pub options: Vec<String>,
    pub required_by: Vec<String>,
    pub unit_name: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolvedNetworkMountPlan {
    pub location: String,
    pub host: String,
    pub mounts: Vec<ResolvedNetworkMount>,
}

impl ResolvedNetworkMountPlan {
    pub fn print(&self) {
        println!(
            "Network mount plan for '{}' at '{}':",
            self.host, self.location
        );
        for mount in &self.mounts {
            println!(
                "{}\t{}:{:?}\t{}:{} -> {}\trequired_by={}",
                mount.id,
                mount.server_host,
                mount.server_ipv4,
                mount.server_ipv4,
                mount.export,
                mount.mount_path.display(),
                mount.required_by.join(",")
            );
        }
    }
}

pub fn resolve_network_mount_plan(
    bundle: &ConfigBundle,
    host_name: &str,
) -> Result<ResolvedNetworkMountPlan> {
    bundle
        .inventory
        .hosts
        .get(host_name)
        .with_context(|| format!("unknown host '{host_name}'"))?;

    let mut mounts = Vec::new();
    for (id, mount) in &bundle.mounts.mounts {
        if mount.host != host_name {
            continue;
        }

        let location_server = bundle
            .location
            .hosts
            .get(&mount.server_host)
            .with_context(|| {
                format!(
                    "network mount '{id}' server '{}' is not attached at location '{}'",
                    mount.server_host, bundle.location.name
                )
            })?;
        let server_ipv4 = location_server
            .interfaces
            .get("lan")
            .and_then(|interface| interface.ipv4)
            .or_else(|| {
                location_server
                    .interfaces
                    .values()
                    .find_map(|interface| interface.ipv4)
            })
            .with_context(|| {
                format!(
                    "network mount '{id}' server '{}' has no IPv4 address",
                    mount.server_host
                )
            })?;

        let mount_path = PathBuf::from(&mount.mount_path);
        if !mount_path.is_absolute() {
            anyhow::bail!(
                "network mount '{id}' path '{}' must be absolute",
                mount.mount_path
            );
        }

        mounts.push(ResolvedNetworkMount {
            id: id.clone(),
            host: mount.host.clone(),
            provider: mount.provider,
            server_host: mount.server_host.clone(),
            server_ipv4,
            export: mount.export.clone(),
            unit_name: systemd_mount_unit(&mount.mount_path)?,
            mount_path,
            options: mount.options.clone(),
            required_by: mount.required_by.clone(),
        });
    }
    mounts.sort_by(|left, right| left.id.cmp(&right.id));

    Ok(ResolvedNetworkMountPlan {
        location: bundle.location.name.clone(),
        host: host_name.to_owned(),
        mounts,
    })
}

fn systemd_mount_unit(path: &str) -> Result<String> {
    let trimmed = path.trim_matches('/');
    if trimmed.is_empty() {
        anyhow::bail!("refusing to create a systemd mount unit for '/'");
    }
    Ok(format!(
        "{}.mount",
        trimmed.replace('-', "\\x2d").replace('/', "-")
    ))
}
