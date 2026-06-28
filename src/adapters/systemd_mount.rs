use anyhow::{Context, Result};

use crate::adapter::artifact::collect_artifacts;
use crate::adapter::{
    Adapter, AdapterContext, AdapterId, AdapterMetadata, AdapterOutput, Capability,
};
use crate::render::systemd_mount::render_systemd_mounts;

const CAPABILITIES: &[Capability] = &[Capability::NetworkMounts];

#[derive(Debug, Default, Clone, Copy)]
pub struct SystemdMountAdapter;

impl Adapter for SystemdMountAdapter {
    fn metadata(&self) -> AdapterMetadata {
        AdapterMetadata {
            id: AdapterId::new("systemd-mounts"),
            name: "systemd network mounts",
            description: "Render systemd network mount units and dependent service drop-ins",
            capabilities: CAPABILITIES,
        }
    }

    fn validate(&self, context: &AdapterContext<'_, '_>) -> Result<()> {
        let host = context
            .target_host
            .context("systemd mount adapter requires --host")?;
        context.plan.network_mounts(host).map(|_| ())
    }

    fn render(&self, context: &AdapterContext<'_, '_>) -> Result<AdapterOutput> {
        let host = context
            .target_host
            .context("systemd mount adapter requires --host")?;
        let plan = context.plan.network_mounts(host)?;
        let root = render_systemd_mounts(&plan, context.plan.paths())?;
        let artifacts = collect_artifacts(&root)?;
        Ok(AdapterOutput {
            adapter: self.metadata().id,
            root,
            target_host: Some(host.to_owned()),
            artifacts,
        })
    }
}
