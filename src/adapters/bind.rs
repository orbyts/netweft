use anyhow::Result;

use crate::adapter::{
    Adapter, AdapterContext, AdapterId, AdapterMetadata, AdapterOutput, Capability,
};
use crate::plan::dns::resolve_dns_plan;
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

    fn validate(&self, context: &AdapterContext<'_>) -> Result<()> {
        resolve_dns_plan(context.bundle).map(|_| ())
    }

    fn render(&self, context: &AdapterContext<'_>) -> Result<AdapterOutput> {
        let plan = resolve_dns_plan(context.bundle)?;
        let output = context
            .paths
            .generated_dir
            .join(&context.bundle.location.name)
            .join("bind");
        let root = render_bind(&plan, &output)?;

        Ok(AdapterOutput {
            adapter: self.metadata().id,
            root,
        })
    }
}
