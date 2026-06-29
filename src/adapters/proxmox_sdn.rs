use anyhow::{Context, Result};

use crate::adapter::artifact::collect_artifacts;
use crate::adapter::{
    Adapter, AdapterContext, AdapterId, AdapterMetadata, AdapterOutput, Capability,
};
use crate::render::proxmox_sdn::render_proxmox_sdn;

const CAPABILITIES: &[Capability] = &[Capability::ProxmoxSdn];

#[derive(Debug, Default, Clone, Copy)]
pub struct ProxmoxSdnAdapter;

impl Adapter for ProxmoxSdnAdapter {
    fn metadata(&self) -> AdapterMetadata {
        AdapterMetadata {
            id: AdapterId::new("proxmox-sdn"),
            name: "Proxmox SDN",
            description: "Render Proxmox SDN zone, VNet, and subnet reconciliation",
            capabilities: CAPABILITIES,
        }
    }

    fn validate(&self, context: &AdapterContext<'_, '_>) -> Result<()> {
        let host = context
            .target_host
            .context("Proxmox SDN adapter requires --host")?;
        context.plan.proxmox_sdn(host)?;
        Ok(())
    }

    fn render(&self, context: &AdapterContext<'_, '_>) -> Result<AdapterOutput> {
        self.validate(context)?;

        let host = context
            .target_host
            .context("Proxmox SDN adapter requires --host")?;
        let plan = context.plan.proxmox_sdn(host)?;
        let root = render_proxmox_sdn(&plan, context.plan.paths())?;
        let artifacts = collect_artifacts(&root)?;

        Ok(AdapterOutput {
            adapter: self.metadata().id,
            root,
            target_host: Some(host.to_owned()),
            artifacts,
        })
    }
}
