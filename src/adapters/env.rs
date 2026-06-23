use anyhow::{Context, Result};

use crate::adapter::{
    Adapter, AdapterContext, AdapterId, AdapterMetadata, AdapterOutput, Capability,
};
use crate::plan::env::resolve_env_plan;
use crate::render::env::render_env;

const CAPABILITIES: &[Capability] = &[Capability::HostEnvironment];

/// Official host environment adapter.
#[derive(Debug, Default, Clone, Copy)]
pub struct EnvironmentAdapter;

impl EnvironmentAdapter {
    fn target_host<'a>(&self, context: &'a AdapterContext<'_>) -> Result<&'a str> {
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

    fn validate(&self, context: &AdapterContext<'_>) -> Result<()> {
        let host = self.target_host(context)?;
        resolve_env_plan(context.bundle, context.paths, host).map(|_| ())
    }

    fn render(&self, context: &AdapterContext<'_>) -> Result<AdapterOutput> {
        let host = self.target_host(context)?;
        let plan = resolve_env_plan(context.bundle, context.paths, host)?;
        let root = render_env(&plan, context.paths)?;

        Ok(AdapterOutput {
            adapter: self.metadata().id,
            root,
        })
    }
}
