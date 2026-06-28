//! Official adapters shipped with the Netweft CLI.

pub mod bind;
pub mod env;
pub mod nas_permission;
pub mod nginx;
pub mod proxmox;
pub mod proxmox_storage;
pub mod systemd_mount;

use anyhow::Result;

use crate::adapter::registry::AdapterRegistry;

/// Build the registry used by the reference CLI.
pub fn builtin_registry() -> Result<AdapterRegistry> {
    let mut registry = AdapterRegistry::new();
    registry.register(bind::BindAdapter)?;
    registry.register(env::EnvironmentAdapter)?;
    registry.register(nginx::NginxAdapter)?;
    registry.register(proxmox::ProxmoxAdapter)?;
    registry.register(systemd_mount::SystemdMountAdapter)?;
    registry.register(nas_permission::NasPermissionAdapter)?;
    registry.register(proxmox_storage::ProxmoxStorageAdapter)?;
    Ok(registry)
}
