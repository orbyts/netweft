use std::net::Ipv4Addr;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};

use crate::model::{ConfigBundle, ProxmoxStorageProvider};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedProxmoxStorage {
    pub id: String,
    pub host: String,
    pub storage_id: String,
    pub provider: ProxmoxStorageProvider,
    pub server_host: String,
    pub server_ipv4: Ipv4Addr,
    pub export: String,
    pub mount_path: PathBuf,
    pub options: Vec<String>,
    pub content: Vec<String>,
    pub prune_backups: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolvedProxmoxStoragePlan {
    pub location: String,
    pub host: String,
    pub storages: Vec<ResolvedProxmoxStorage>,
}

impl ResolvedProxmoxStoragePlan {
    pub fn print(&self) {
        println!(
            "Proxmox storage plan for '{}' at '{}':",
            self.host, self.location
        );
        for storage in &self.storages {
            println!(
                "{}\tstorage_id={}\tserver={}({})\texport={}\tpath={}\tcontent={}\toptions={}",
                storage.id,
                storage.storage_id,
                storage.server_host,
                storage.server_ipv4,
                storage.export,
                storage.mount_path.display(),
                storage.content.join(","),
                storage.options.join(","),
            );
        }
    }
}

pub fn resolve_proxmox_storage_plan(
    bundle: &ConfigBundle,
    host_name: &str,
) -> Result<ResolvedProxmoxStoragePlan> {
    bundle
        .inventory
        .hosts
        .get(host_name)
        .with_context(|| format!("unknown host '{host_name}'"))?;

    let mut storages = Vec::new();
    for (id, storage) in &bundle.proxmox_storages.storages {
        if storage.host != host_name {
            continue;
        }

        let location_server = bundle
            .location
            .hosts
            .get(&storage.server_host)
            .with_context(|| {
                format!(
                    "Proxmox storage '{id}' server '{}' is not attached at location '{}'",
                    storage.server_host, bundle.location.name
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
                    "Proxmox storage '{id}' server '{}' has no IPv4 address",
                    storage.server_host
                )
            })?;

        let mount_path = PathBuf::from(&storage.mount_path);
        if !mount_path.is_absolute() {
            bail!(
                "Proxmox storage '{id}' path '{}' must be absolute",
                storage.mount_path
            );
        }

        storages.push(ResolvedProxmoxStorage {
            id: id.clone(),
            host: storage.host.clone(),
            storage_id: storage.storage_id.clone(),
            provider: storage.provider,
            server_host: storage.server_host.clone(),
            server_ipv4,
            export: storage.export.clone(),
            mount_path,
            options: storage.options.clone(),
            content: storage.content.clone(),
            prune_backups: storage.prune_backups.clone(),
        });
    }
    storages.sort_by(|left, right| left.id.cmp(&right.id));

    Ok(ResolvedProxmoxStoragePlan {
        location: bundle.location.name.clone(),
        host: host_name.to_owned(),
        storages,
    })
}
