//! Provider-neutral resolution boundary shared by adapters.
//!
//! The resolver owns the distinction between declared configuration,
//! observations, and tool-specific rendering. Adapters ask this object for
//! resolved plans instead of reinterpreting TOML themselves.

use anyhow::Result;

use crate::model::ConfigBundle;
use crate::observe::ObservationSet;
use crate::paths::NetweftPaths;
use crate::plan::dns::{ResolvedDnsPlan, resolve_dns_plan};
use crate::plan::env::{ResolvedEnvPlan, resolve_env_plan};
use crate::plan::guest::{ResolvedGuestPlan, resolve_guest_plan};
use crate::plan::host_network::{ResolvedHostNetworkPlan, resolve_host_network_plan};
use crate::plan::nas_permission::{ResolvedNasPermissionPlan, resolve_nas_permission_plan};
use crate::plan::network_mount::{ResolvedNetworkMountPlan, resolve_network_mount_plan};
use crate::plan::proxmox_storage::{ResolvedProxmoxStoragePlan, resolve_proxmox_storage_plan};
use crate::plan::proxy::{ResolvedProxyPlan, resolve_proxy_plan};

/// Shared resolution input for all adapters.
#[derive(Debug)]
pub struct ResolvedPlan<'a> {
    config: &'a ConfigBundle,
    paths: &'a NetweftPaths,
    observations: ObservationSet,
}

impl<'a> ResolvedPlan<'a> {
    /// Build a deterministic plan using declared configuration only.
    pub fn new(config: &'a ConfigBundle, paths: &'a NetweftPaths) -> Self {
        Self {
            config,
            paths,
            observations: ObservationSet::empty(),
        }
    }

    /// Attach explicitly collected observations.
    pub fn with_observations(mut self, observations: ObservationSet) -> Self {
        self.observations = observations;
        self
    }

    pub fn config(&self) -> &ConfigBundle {
        self.config
    }

    pub fn paths(&self) -> &NetweftPaths {
        self.paths
    }

    pub fn observations(&self) -> &ObservationSet {
        &self.observations
    }

    pub fn dns(&self) -> Result<ResolvedDnsPlan> {
        resolve_dns_plan(self.config)
    }

    pub fn proxies(&self) -> Result<ResolvedProxyPlan> {
        resolve_proxy_plan(self.config)
    }

    pub fn host_environment(&self, host: &str) -> Result<ResolvedEnvPlan> {
        resolve_env_plan(self.config, self.paths, host)
    }

    pub fn host_network(&self, host: &str) -> Result<ResolvedHostNetworkPlan> {
        resolve_host_network_plan(self.config, host)
    }

    pub fn guests(&self) -> Result<ResolvedGuestPlan> {
        resolve_guest_plan(self.config)
    }

    pub fn network_mounts(&self, host: &str) -> Result<ResolvedNetworkMountPlan> {
        resolve_network_mount_plan(self.config, host)
    }

    pub fn nas_permissions(&self, nas: Option<&str>) -> Result<ResolvedNasPermissionPlan> {
        resolve_nas_permission_plan(self.config, nas)
    }

    pub fn proxmox_storage(&self, host: &str) -> Result<ResolvedProxmoxStoragePlan> {
        resolve_proxmox_storage_plan(self.config, host)
    }
}
