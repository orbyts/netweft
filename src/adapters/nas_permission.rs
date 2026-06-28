use anyhow::{Context, Result};

use crate::adapter::artifact::collect_artifacts;
use crate::adapter::{
    Adapter, AdapterContext, AdapterId, AdapterMetadata, AdapterOutput, Capability,
};
use crate::render::nas_permission::render_nas_permissions;

const CAPABILITIES: &[Capability] = &[Capability::NasPermissions];

#[derive(Debug, Default, Clone, Copy)]
pub struct NasPermissionAdapter;

impl Adapter for NasPermissionAdapter {
    fn metadata(&self) -> AdapterMetadata {
        AdapterMetadata {
            id: AdapterId::new("synology-nfs-permissions"),
            name: "Synology NFS permissions",
            description: "Render location-resolved Synology NFS permission action plans",
            capabilities: CAPABILITIES,
        }
    }

    fn validate(&self, context: &AdapterContext<'_, '_>) -> Result<()> {
        let nas = context
            .target_host
            .context("Synology NFS permission adapter requires --nas")?;
        context.plan.nas_permissions(Some(nas)).map(|_| ())
    }

    fn render(&self, context: &AdapterContext<'_, '_>) -> Result<AdapterOutput> {
        let nas = context
            .target_host
            .context("Synology NFS permission adapter requires --nas")?;
        let plan = context.plan.nas_permissions(Some(nas))?;
        let root = render_nas_permissions(&plan, context.plan.paths())?;
        let artifacts = collect_artifacts(&root)?;
        Ok(AdapterOutput {
            adapter: self.metadata().id,
            root,
            target_host: Some(nas.to_owned()),
            artifacts,
        })
    }
}
