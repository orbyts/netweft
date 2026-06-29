use anyhow::{Context, Result};

use crate::adapter::artifact::collect_artifacts;
use crate::adapter::{
    Adapter, AdapterContext, AdapterId, AdapterMetadata, AdapterOutput, Capability,
};
use crate::render::docker::render_docker;

const CAPABILITIES: &[Capability] = &[Capability::DockerNetworking];

#[derive(Debug, Default, Clone, Copy)]
pub struct DockerAdapter;

impl Adapter for DockerAdapter {
    fn metadata(&self) -> AdapterMetadata {
        AdapterMetadata {
            id: AdapterId::new("docker"),
            name: "Docker networking",
            description: "Render Docker daemon and named bridge network reconciliation",
            capabilities: CAPABILITIES,
        }
    }

    fn validate(&self, context: &AdapterContext<'_, '_>) -> Result<()> {
        let host = context
            .target_host
            .context("Docker adapter requires --host")?;
        context.plan.docker(host)?;
        Ok(())
    }

    fn render(&self, context: &AdapterContext<'_, '_>) -> Result<AdapterOutput> {
        self.validate(context)?;
        let host = context
            .target_host
            .context("Docker adapter requires --host")?;
        let plan = context.plan.docker(host)?;
        let root = render_docker(&plan, context.plan.paths())?;
        let artifacts = collect_artifacts(&root)?;

        Ok(AdapterOutput {
            adapter: self.metadata().id,
            root,
            target_host: Some(host.to_owned()),
            artifacts,
        })
    }
}
