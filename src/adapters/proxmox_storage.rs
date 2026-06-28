use anyhow::{Context, Result};

use crate::adapter::artifact::collect_artifacts;
use crate::adapter::{
    Adapter, AdapterContext, AdapterId, AdapterMetadata, AdapterOutput, Capability,
};
use crate::render::proxmox_storage::render_proxmox_storage;

const CAPABILITIES: &[Capability] = &[Capability::ProxmoxStorage];

#[derive(Debug, Default, Clone, Copy)]
pub struct ProxmoxStorageAdapter;

impl Adapter for ProxmoxStorageAdapter {
    fn metadata(&self) -> AdapterMetadata {
        AdapterMetadata {
            id: AdapterId::new("proxmox-storage"),
            name: "Proxmox storage",
            description: "Render identity-resolved Proxmox storage configuration and deployment scripts",
            capabilities: CAPABILITIES,
        }
    }

    fn validate(&self, context: &AdapterContext<'_, '_>) -> Result<()> {
        let host = context
            .target_host
            .context("Proxmox storage adapter requires --host")?;
        context.plan.proxmox_storage(host).map(|_| ())
    }

    fn render(&self, context: &AdapterContext<'_, '_>) -> Result<AdapterOutput> {
        let host = context
            .target_host
            .context("Proxmox storage adapter requires --host")?;
        let plan = context.plan.proxmox_storage(host)?;
        let root = render_proxmox_storage(&plan, context.plan.paths())?;
        let artifacts = collect_artifacts(&root)?;
        Ok(AdapterOutput {
            adapter: self.metadata().id,
            root,
            target_host: Some(host.to_owned()),
            artifacts,
        })
    }
}
