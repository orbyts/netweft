use std::net::IpAddr;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};

use crate::model::id::{CertificateId, DomainName, HostId, ProxyId, ServiceId};
use crate::model::{ConfigBundle, ServiceKind, UpstreamScheme};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListenerProtocol {
    Http,
    Https,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ResolvedListener {
    pub protocol: ListenerProtocol,
    pub port: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ResolvedUpstream {
    pub scheme: UpstreamScheme,
    pub address: IpAddr,
    pub port: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ResolvedCertificate {
    pub id: CertificateId,
    pub domains: Vec<DomainName>,
    pub certificate_path: PathBuf,
    pub private_key_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ResolvedTls {
    pub certificate: Option<ResolvedCertificate>,
    pub force_https: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ResolvedProxy {
    pub id: ProxyId,
    pub service: ServiceId,
    pub proxy_service: ServiceId,
    pub target_host: HostId,
    pub domains: Vec<DomainName>,
    pub listeners: Vec<ResolvedListener>,
    pub upstream: ResolvedUpstream,
    pub tls: Option<ResolvedTls>,
    pub websocket: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct ResolvedProxyPlan {
    pub proxies: Vec<ResolvedProxy>,
}

impl ResolvedProxyPlan {
    pub fn print(&self) {
        for proxy in &self.proxies {
            let listeners = proxy
                .listeners
                .iter()
                .map(|listener| format!("{}:{}", listener.protocol, listener.port))
                .collect::<Vec<_>>()
                .join(",");
            println!(
                "{}\tdomains={}\tlisteners={}\tupstream={}://{}:{}\tproxy={}\thost={}\twebsocket={}",
                proxy.id,
                proxy.domains.iter().map(ToString::to_string).collect::<Vec<_>>().join(","),
                listeners,
                proxy.upstream.scheme,
                proxy.upstream.address,
                proxy.upstream.port,
                proxy.proxy_service,
                proxy.target_host,
                proxy.websocket,
            );
        }
    }
}

pub fn resolve_proxy_plan(bundle: &ConfigBundle) -> Result<ResolvedProxyPlan> {
    let mut proxies = Vec::new();

    for (service_name, service) in &bundle.services.services {
        let Some(web) = service.web.as_ref() else {
            continue;
        };
        if !service.enabled {
            continue;
        }
        if web.access != "reverse-proxy" {
            bail!("service '{service_name}' has unsupported web access '{}'", web.access);
        }

        let proxy_service = bundle
            .services
            .services
            .get(&web.proxy)
            .with_context(|| format!("service '{service_name}' references unknown proxy service '{}'", web.proxy))?;
        if !matches!(&proxy_service.kind, ServiceKind::ReverseProxy) {
            bail!("service '{service_name}' references '{}' as a proxy, but it is not a reverse-proxy service", web.proxy);
        }
        if !proxy_service.enabled {
            bail!("service '{service_name}' references disabled proxy service '{}'", web.proxy);
        }

        let address = service
            .address
            .as_ref()
            .and_then(|address| address.ipv4)
            .with_context(|| format!("proxied service '{service_name}' requires an IPv4 address"))?;
        if web.container_port == 0 {
            bail!("proxied service '{service_name}' uses invalid upstream port 0");
        }

        let domain = DomainName::new(&web.domain)
            .with_context(|| format!("service '{service_name}' has invalid proxy domain"))?;
        if domain.as_str().starts_with("*.") {
            bail!("service '{service_name}' cannot use a wildcard proxy domain");
        }
        let tls = if web.tls {
            let certificate = match web.certificate.as_deref() {
                Some(certificate_name) => {
                    let reference = bundle.services.certificates.get(certificate_name).with_context(|| {
                        format!("service '{service_name}' references unknown certificate '{certificate_name}'")
                    })?;
                    let domains = reference
                        .domains
                        .iter()
                        .map(|domain| DomainName::new(domain.as_str()))
                        .collect::<Result<Vec<_>>>()?;
                    if !domains.iter().any(|covered| domain_matches(covered, &domain)) {
                        bail!("certificate '{certificate_name}' does not cover domain '{domain}'");
                    }
                    Some(ResolvedCertificate {
                        id: CertificateId::new(certificate_name)?,
                        domains,
                        certificate_path: absolute_path(&reference.certificate_path, certificate_name)?,
                        private_key_path: absolute_path(&reference.private_key_path, certificate_name)?,
                    })
                }
                None => None,
            };
            Some(ResolvedTls {
                certificate,
                force_https: web.force_https,
            })
        } else {
            if web.certificate.is_some() || web.force_https {
                bail!("service '{service_name}' configures certificate or force-HTTPS without TLS");
            }
            None
        };

        proxies.push(ResolvedProxy {
            id: ProxyId::new(service_name)?,
            service: ServiceId::new(service_name)?,
            proxy_service: ServiceId::new(&web.proxy)?,
            target_host: HostId::new(&proxy_service.host)?,
            domains: vec![domain],
            listeners: vec![ResolvedListener {
                protocol: if web.tls { ListenerProtocol::Https } else { ListenerProtocol::Http },
                port: if web.tls { 443 } else { 80 },
            }],
            upstream: ResolvedUpstream {
                scheme: web.scheme,
                address: IpAddr::V4(address),
                port: web.container_port,
            },
            tls,
            websocket: web.websocket,
        });
    }

    proxies.sort_by(|left, right| left.id.cmp(&right.id));
    let mut domains = std::collections::BTreeSet::new();
    for proxy in &proxies {
        for domain in &proxy.domains {
            if !domains.insert(domain.clone()) {
                bail!("proxy domain '{domain}' is declared more than once");
            }
        }
    }

    Ok(ResolvedProxyPlan { proxies })
}

fn absolute_path(value: &str, certificate: &str) -> Result<PathBuf> {
    let path = PathBuf::from(value);
    if !path.is_absolute() {
        bail!("certificate '{certificate}' path '{}' must be absolute", path.display());
    }
    Ok(path)
}

fn domain_matches(covered: &DomainName, requested: &DomainName) -> bool {
    if covered == requested {
        return true;
    }
    covered
        .as_str()
        .strip_prefix("*.")
        .is_some_and(|suffix| requested.as_str().strip_suffix(suffix).is_some_and(|prefix| prefix.ends_with('.') && !prefix[..prefix.len() - 1].contains('.')))
}

impl std::fmt::Display for ListenerProtocol {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Http => "http",
            Self::Https => "https",
        })
    }
}

impl std::fmt::Display for UpstreamScheme {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Http => "http",
            Self::Https => "https",
        })
    }
}
