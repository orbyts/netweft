use anyhow::Result;

use crate::adapter::artifact::collect_artifacts;
use crate::adapter::{
    Adapter, AdapterContext, AdapterId, AdapterMetadata, AdapterOutput, Capability,
};
use crate::render::bind::render_bind;

const CAPABILITIES: &[Capability] = &[
    Capability::AuthoritativeDns,
    Capability::RecursiveDns,
];

/// Official BIND 9 configuration adapter.
#[derive(Debug, Default, Clone, Copy)]
pub struct BindAdapter;

impl Adapter for BindAdapter {
    fn metadata(&self) -> AdapterMetadata {
        AdapterMetadata {
            id: AdapterId::new("bind"),
            name: "BIND 9",
            description: "Render authoritative and recursive BIND configuration",
            capabilities: CAPABILITIES,
        }
    }

    fn validate(&self, context: &AdapterContext<'_, '_>) -> Result<()> {
        context.plan.dns().map(|_| ())
    }

    fn render(&self, context: &AdapterContext<'_, '_>) -> Result<AdapterOutput> {
        let plan = context.plan.dns()?;
        let output = context
            .plan
            .paths()
            .generated_dir
            .join(&context.plan.config().location.name)
            .join("bind");
        let root = render_bind(&plan, &output)?;

        let artifacts = collect_artifacts(&root)?;
        Ok(AdapterOutput {
            adapter: self.metadata().id,
            root,
            target_host: Some(
                context.plan.config().services.services
                    [&context.plan.config().dns.dns.service]
                    .host
                    .clone(),
            ),
            artifacts,
        })
    }
}
