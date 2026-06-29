use anyhow::{Context, Result, bail};

use crate::adapter::artifact::collect_artifacts;
use crate::adapter::{
    Adapter, AdapterContext, AdapterId, AdapterMetadata, AdapterOutput, Capability,
};
use crate::model::HostOsNetworkProvider;
use crate::render::netplan::render_netplan;

const CAPABILITIES: &[Capability] = &[Capability::HostNetworking];

#[derive(Debug, Default, Clone, Copy)]
pub struct NetplanAdapter;

impl Adapter for NetplanAdapter {
    fn metadata(&self) -> AdapterMetadata {
        AdapterMetadata {
            id: AdapterId::new("netplan"),
            name: "Netplan",
            description: "Render Ubuntu Netplan configuration with guarded deployment and rollback",
            capabilities: CAPABILITIES,
        }
    }

    fn validate(&self, context: &AdapterContext<'_, '_>) -> Result<()> {
        let host = context
            .target_host
            .context("Netplan adapter requires --host")?;
        let plan = context.plan.os_network(host)?;
        if plan.provider != HostOsNetworkProvider::Netplan {
            bail!("host '{host}' does not use the netplan provider");
        }
        Ok(())
    }

    fn render(&self, context: &AdapterContext<'_, '_>) -> Result<AdapterOutput> {
        self.validate(context)?;

        let host = context
            .target_host
            .context("Netplan adapter requires --host")?;
        let plan = context.plan.os_network(host)?;
        let root = render_netplan(&plan, context.plan.paths())?;
        let artifacts = collect_artifacts(&root)?;

        Ok(AdapterOutput {
            adapter: self.metadata().id,
            root,
            target_host: Some(host.to_owned()),
            artifacts,
        })
    }
}
