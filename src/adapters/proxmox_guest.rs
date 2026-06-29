use anyhow::{Context, Result, bail};

use crate::adapter::artifact::collect_artifacts;
use crate::adapter::{
    Adapter, AdapterContext, AdapterId, AdapterMetadata, AdapterOutput, Capability,
};
use crate::render::proxmox_guest::render_proxmox_guests;

const CAPABILITIES: &[Capability] = &[Capability::ProxmoxGuests];

#[derive(Debug, Default, Clone, Copy)]
pub struct ProxmoxGuestAdapter;

impl Adapter for ProxmoxGuestAdapter {
    fn metadata(&self) -> AdapterMetadata {
        AdapterMetadata {
            id: AdapterId::new("proxmox-guests"),
            name: "Proxmox guests",
            description: "Render safe Proxmox VM and LXC reconciliation scripts",
            capabilities: CAPABILITIES,
        }
    }

    fn validate(&self, context: &AdapterContext<'_, '_>) -> Result<()> {
        let host = context
            .target_host
            .context("Proxmox guest adapter requires --host")?;
        let plan = context.plan.guests()?;

        if !context.plan.config().inventory.hosts.contains_key(host) {
            bail!("unknown Proxmox host '{host}'");
        }

        if !plan.guests.iter().any(|guest| guest.host == host) {
            bail!("host '{host}' has no configured Proxmox guests");
        }

        Ok(())
    }

    fn render(&self, context: &AdapterContext<'_, '_>) -> Result<AdapterOutput> {
        self.validate(context)?;

        let host = context
            .target_host
            .context("Proxmox guest adapter requires --host")?;
        let plan = context.plan.guests()?;
        let root = render_proxmox_guests(&plan, host, context.plan.paths())?;
        let artifacts = collect_artifacts(&root)?;

        Ok(AdapterOutput {
            adapter: self.metadata().id,
            root,
            target_host: Some(host.to_owned()),
            artifacts,
        })
    }
}
