use anyhow::{Context, Result};

use crate::adapter::artifact::collect_artifacts;
use crate::adapter::{
    Adapter, AdapterContext, AdapterId, AdapterMetadata, AdapterOutput, Capability,
};
use crate::render::ssh::render_ssh;

const CAPABILITIES: &[Capability] = &[Capability::SshClientConfig];

#[derive(Debug, Default, Clone, Copy)]
pub struct SshAdapter;

impl Adapter for SshAdapter {
    fn metadata(&self) -> AdapterMetadata {
        AdapterMetadata {
            id: AdapterId::new("ssh"),
            name: "SSH client",
            description: "Render location-resolved OpenSSH client configuration",
            capabilities: CAPABILITIES,
        }
    }

    fn validate(&self, context: &AdapterContext<'_, '_>) -> Result<()> {
        let client = context
            .target_host
            .context("SSH adapter requires --client")?;
        context.plan.ssh(client).map(|_| ())
    }

    fn render(&self, context: &AdapterContext<'_, '_>) -> Result<AdapterOutput> {
        let client = context
            .target_host
            .context("SSH adapter requires --client")?;
        let plan = context.plan.ssh(client)?;
        let root = render_ssh(&plan, context.plan.paths())?;
        let artifacts = collect_artifacts(&root)?;
        Ok(AdapterOutput {
            adapter: self.metadata().id,
            root,
            target_host: Some(client.to_owned()),
            artifacts,
        })
    }
}
