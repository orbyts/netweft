use anyhow::{Context, Result, bail};

use crate::adapter::artifact::collect_artifacts;
use crate::adapter::{
    Adapter, AdapterContext, AdapterId, AdapterMetadata, AdapterOutput, Capability,
};
use crate::model::HostNetworkProvider;
use crate::plan::host_network::ResolvedHostNetworkPlan;
use crate::render::proxmox::render_proxmox;

const CAPABILITIES: &[Capability] = &[Capability::HostNetworking];

#[derive(Debug, Default, Clone, Copy)]
pub struct ProxmoxAdapter;

impl Adapter for ProxmoxAdapter {
    fn metadata(&self) -> AdapterMetadata {
        AdapterMetadata {
            id: AdapterId::new("proxmox"),
            name: "Proxmox",
            description: "Render Proxmox ifupdown2 host networking",
            capabilities: CAPABILITIES,
        }
    }

    fn validate(&self, context: &AdapterContext<'_, '_>) -> Result<()> {
        let (_, plan) = plan_for_context(context)?;
        validate_plan(&plan)
    }

    fn render(&self, context: &AdapterContext<'_, '_>) -> Result<AdapterOutput> {
        let (target_host, plan) = plan_for_context(context)?;

        validate_plan(&plan)?;

        let root = context
            .plan
            .paths()
            .generated_dir
            .join(&context.plan.config().location.name)
            .join("hosts")
            .join(&target_host)
            .join("proxmox");

        let root = render_proxmox(&plan, &root)?;
        let artifacts = collect_artifacts(&root)?;

        Ok(AdapterOutput {
            adapter: self.metadata().id,
            root,
            target_host: Some(target_host),
            artifacts,
        })
    }
}

fn plan_for_context(context: &AdapterContext<'_, '_>) -> Result<(String, ResolvedHostNetworkPlan)> {
    let host = context
        .target_host
        .context("Proxmox adapter requires --host")?
        .to_owned();

    let plan = context.plan.host_network(&host)?;

    Ok((host, plan))
}

fn validate_plan(plan: &ResolvedHostNetworkPlan) -> Result<()> {
    if plan.provider != HostNetworkProvider::ProxmoxIfupdown2 {
        bail!(
            "host '{}' does not use the proxmox-ifupdown2 provider",
            plan.host
        );
    }

    if plan.bridges.is_empty() {
        bail!("host '{}' has no network bridges", plan.host);
    }

    let management_bridges = plan
        .bridges
        .iter()
        .filter(|bridge| {
            bridge.location_interface.as_deref() == Some(plan.management_interface.as_str())
        })
        .count();

    if management_bridges != 1 {
        bail!(
            "host '{}' must resolve exactly one management bridge",
            plan.host
        );
    }

    Ok(())
}
