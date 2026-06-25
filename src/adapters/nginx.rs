use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::adapter::artifact::collect_artifacts;
use crate::adapter::{
    Adapter, AdapterContext, AdapterId, AdapterMetadata, AdapterOutput, Capability,
};
use crate::plan::proxy::ResolvedProxyPlan;
use crate::render::nginx::render_nginx;

const CAPABILITIES: &[Capability] = &[Capability::ReverseProxy, Capability::CertificateIntent];

/// Official native Nginx configuration adapter.
#[derive(Debug, Default, Clone, Copy)]
pub struct NginxAdapter;

impl Adapter for NginxAdapter {
    fn metadata(&self) -> AdapterMetadata {
        AdapterMetadata {
            id: AdapterId::new("nginx"),
            name: "Nginx",
            description: "Render native Nginx reverse-proxy configuration",
            capabilities: CAPABILITIES,
        }
    }

    fn validate(&self, context: &AdapterContext<'_, '_>) -> Result<()> {
        let (plan, _) = plan_for_context(context)?;
        validate_plan(&plan)
    }

    fn render(&self, context: &AdapterContext<'_, '_>) -> Result<AdapterOutput> {
        let (plan, target_host) = plan_for_context(context)?;
        validate_plan(&plan)?;

        let root = context
            .plan
            .paths()
            .generated_dir
            .join(&context.plan.config().location.name)
            .join("hosts")
            .join(&target_host)
            .join("nginx");
        let root = render_nginx(&plan, &root)?;
        let artifacts = collect_artifacts(&root)?;

        Ok(AdapterOutput {
            adapter: self.metadata().id,
            root,
            target_host: Some(target_host),
            artifacts,
        })
    }
}

pub fn validate_nginx_config(root: &Path, executable: &Path) -> Result<()> {
    let config = root.join("nginx.conf");
    if !config.is_file() {
        bail!("Nginx configuration does not exist: {}", config.display());
    }

    let status = Command::new(executable)
        .arg("-t")
        .arg("-p")
        .arg(prefix_argument(root))
        .arg("-c")
        .arg(&config)
        .status()
        .with_context(|| format!("failed to execute {}", executable.display()))?;

    if !status.success() {
        bail!("Nginx configuration validation failed with status {status}");
    }
    Ok(())
}

fn prefix_argument(root: &Path) -> PathBuf {
    let mut value = root.as_os_str().to_os_string();
    value.push(std::path::MAIN_SEPARATOR.to_string());
    PathBuf::from(value)
}

fn plan_for_context(context: &AdapterContext<'_, '_>) -> Result<(ResolvedProxyPlan, String)> {
    let mut plan = context.plan.proxies()?;
    let target_host = match context.target_host {
        Some(host) => {
            plan.proxies
                .retain(|proxy| proxy.target_host.as_str() == host);
            host.to_owned()
        }
        None => {
            let hosts = plan
                .proxies
                .iter()
                .map(|proxy| proxy.target_host.as_str())
                .collect::<std::collections::BTreeSet<_>>();
            let mut hosts = hosts.into_iter();
            let Some(host) = hosts.next() else {
                bail!("Nginx adapter has no resolved proxy entries");
            };
            if hosts.next().is_some() {
                bail!("Nginx adapter requires --host when proxies target multiple hosts");
            }
            host.to_owned()
        }
    };

    if plan.proxies.is_empty() {
        bail!("Nginx adapter has no proxy entries for host '{target_host}'");
    }
    Ok((plan, target_host))
}

fn validate_plan(plan: &ResolvedProxyPlan) -> Result<()> {
    for proxy in &plan.proxies {
        if proxy
            .tls
            .as_ref()
            .is_some_and(|tls| tls.certificate.is_none())
        {
            bail!(
                "proxy '{}' enables TLS but has no certificate reference for the Nginx adapter",
                proxy.id
            );
        }
    }
    Ok(())
}
