use anyhow::{Context, Result};

use crate::adapter::artifact::collect_artifacts;
use crate::adapter::{
    Adapter, AdapterContext, AdapterId, AdapterMetadata, AdapterOutput, Capability,
};
use crate::render::env::render_env;

const CAPABILITIES: &[Capability] = &[Capability::HostEnvironment];

/// Official host environment adapter.
#[derive(Debug, Default, Clone, Copy)]
pub struct EnvironmentAdapter;

impl EnvironmentAdapter {
    fn target_host<'a, 'plan>(&self, context: &AdapterContext<'a, 'plan>) -> Result<&'a str> {
        context
            .target_host
            .context("environment adapter requires a target host")
    }
}

impl Adapter for EnvironmentAdapter {
    fn metadata(&self) -> AdapterMetadata {
        AdapterMetadata {
            id: AdapterId::new("env"),
            name: "Host environment",
            description: "Render Compose and shell environment files for a host",
            capabilities: CAPABILITIES,
        }
    }

    fn validate(&self, context: &AdapterContext<'_, '_>) -> Result<()> {
        let host = self.target_host(context)?;
        context.plan.host_environment(host).map(|_| ())
    }

    fn render(&self, context: &AdapterContext<'_, '_>) -> Result<AdapterOutput> {
        let host = self.target_host(context)?;
        let plan = context.plan.host_environment(host)?;
        let root = render_env(&plan, context.plan.paths())?;

        let artifacts = collect_artifacts(&root)?;
        Ok(AdapterOutput {
            adapter: self.metadata().id,
            root,
            target_host: Some(host.to_owned()),
            artifacts,
        })
    }
}
