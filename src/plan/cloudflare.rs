use anyhow::{Context, Result, bail};

use crate::model::{ConfigBundle, ExternalIngressMode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedCloudflarePlan {
    pub location: String,
    pub mode: String,
    pub provider_name: String,
    pub zone: String,
    pub zone_id: String,
    pub account_id: String,
    pub tunnel_api_token_env: String,
    pub dns_api_token_env: String,
    pub tunnel_name: String,
    pub connector_host: String,
    pub origin: String,
    pub origin_tls_verify: bool,
    pub hostnames: Vec<String>,
}

impl ResolvedCloudflarePlan {
    pub fn print(&self) {
        println!("Cloudflare ingress for '{}':", self.location);
        println!("  mode: {}", self.mode);
        println!("  provider: {}", self.provider_name);
        println!("  zone: {}", self.zone);
        println!("  connector: {}", self.connector_host);
        println!("  tunnel: {}", self.tunnel_name);
        println!("  origin: {}", self.origin);
        println!("  tunnel-token-env: {}", self.tunnel_api_token_env);
        println!("  dns-token-env: {}", self.dns_api_token_env);
        println!("  direct public IPv4: unused");
        println!("  direct public IPv6: unused");
        for hostname in &self.hostnames {
            println!("  route: {hostname} -> {}", self.origin);
        }
    }
}

pub fn resolve_cloudflare_plan(
    bundle: &ConfigBundle,
    requested_tunnel: Option<&str>,
) -> Result<ResolvedCloudflarePlan> {
    let ingress = bundle
        .location
        .external_ingress
        .as_ref()
        .context("active location has no external_ingress configuration")?;

    match ingress.mode {
        ExternalIngressMode::Disabled => bail!(
            "external ingress is disabled for location '{}'",
            bundle.location.name
        ),
        ExternalIngressMode::CloudflareDirect => bail!(
            "Cloudflare direct-origin mode is modeled but not yet implemented; use cloudflare-tunnel"
        ),
        ExternalIngressMode::CloudflareTunnel => {}
    }

    let tunnel_name = requested_tunnel
        .or(ingress.tunnel.as_deref())
        .context("cloudflare-tunnel mode requires a tunnel")?;
    if let Some(selected) = ingress.tunnel.as_deref()
        && selected != tunnel_name
    {
        bail!(
            "location '{}' selects tunnel '{}', not '{}'",
            bundle.location.name,
            selected,
            tunnel_name
        );
    }

    let provider = bundle
        .cloudflare
        .providers
        .get(&ingress.provider)
        .with_context(|| format!("unknown Cloudflare provider '{}'", ingress.provider))?;
    let tunnel = bundle
        .cloudflare
        .tunnels
        .get(tunnel_name)
        .with_context(|| format!("unknown Cloudflare tunnel '{tunnel_name}'"))?;

    if tunnel.provider != ingress.provider {
        bail!(
            "Cloudflare tunnel '{}' uses provider '{}', but location '{}' selects '{}'",
            tunnel_name,
            tunnel.provider,
            bundle.location.name,
            ingress.provider
        );
    }
    let host = bundle
        .inventory
        .hosts
        .get(&tunnel.connector_host)
        .with_context(|| {
            format!(
                "unknown Cloudflare connector host '{}'",
                tunnel.connector_host
            )
        })?;
    if !host.enabled {
        bail!(
            "Cloudflare connector host '{}' is disabled",
            tunnel.connector_host
        );
    }
    if tunnel.hostnames.is_empty() {
        bail!("Cloudflare tunnel '{tunnel_name}' must publish at least one hostname");
    }

    let mut hostnames = tunnel.hostnames.clone();
    hostnames.sort();
    hostnames.dedup();
    for hostname in &hostnames {
        if hostname == &provider.zone || !hostname.ends_with(&format!(".{}", provider.zone)) {
            bail!(
                "Cloudflare hostname '{}' is not a subdomain of zone '{}'",
                hostname,
                provider.zone
            );
        }
    }

    Ok(ResolvedCloudflarePlan {
        location: bundle.location.name.clone(),
        mode: ingress.mode.as_str().to_owned(),
        provider_name: ingress.provider.clone(),
        zone: provider.zone.clone(),
        zone_id: provider.zone_id.clone(),
        account_id: provider.account_id.clone(),
        tunnel_api_token_env: provider.tunnel_api_token_env.clone(),
        dns_api_token_env: provider.dns_api_token_env.clone(),
        tunnel_name: tunnel_name.to_owned(),
        connector_host: tunnel.connector_host.clone(),
        origin: tunnel.origin.clone(),
        origin_tls_verify: tunnel.origin_tls_verify,
        hostnames,
    })
}
