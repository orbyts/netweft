use crate::adapter::artifact::collect_artifacts;
use crate::adapter::{
    Adapter, AdapterContext, AdapterId, AdapterMetadata, AdapterOutput, Capability,
};
use crate::render::cloudflare::render_cloudflare;
use anyhow::Result;

const CAPABILITIES: &[Capability] = &[
    Capability::ExternalIngress,
    Capability::CloudflareDns,
    Capability::CloudflareTunnel,
];
#[derive(Debug, Default, Clone, Copy)]
pub struct CloudflareAdapter;
impl Adapter for CloudflareAdapter {
    fn metadata(&self) -> AdapterMetadata {
        AdapterMetadata {
            id: AdapterId::new("cloudflare"),
            name: "Cloudflare ingress",
            description: "Render location-aware Cloudflare Tunnel and DNS reconciliation",
            capabilities: CAPABILITIES,
        }
    }
    fn validate(&self, context: &AdapterContext<'_, '_>) -> Result<()> {
        context.plan.cloudflare(context.target_host).map(|_| ())
    }
    fn render(&self, context: &AdapterContext<'_, '_>) -> Result<AdapterOutput> {
        let plan = context.plan.cloudflare(context.target_host)?;
        let root = render_cloudflare(&plan, context.plan.paths())?;
        let artifacts = collect_artifacts(&root)?;
        Ok(AdapterOutput {
            adapter: self.metadata().id,
            root,
            target_host: Some(plan.connector_host),
            artifacts,
        })
    }
}
