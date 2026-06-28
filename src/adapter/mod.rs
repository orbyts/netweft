//! Public adapter SDK and built-in adapter registry.
//!
//! Adapters consume Netweft's validated configuration and resolved plans, then
//! emit tool-specific artifacts. The first SDK version keeps deployment
//! explicit: adapters render files, but do not apply them to remote systems.

pub mod artifact;
pub mod registry;

use std::path::PathBuf;

use artifact::Artifact;

use anyhow::Result;

use crate::resolve::ResolvedPlan;

/// Stable identifier for an adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AdapterId(&'static str);

impl AdapterId {
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl std::fmt::Display for AdapterId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.0)
    }
}

/// Broad capability exposed by an adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Capability {
    AuthoritativeDns,
    RecursiveDns,
    HostEnvironment,
    HostNetworking,
    NetworkMounts,
    ReverseProxy,
    OverlayRouting,
    Firewall,
    CertificateIntent,
}

/// Human- and machine-readable adapter metadata.
#[derive(Debug, Clone, Copy)]
pub struct AdapterMetadata {
    pub id: AdapterId,
    pub name: &'static str,
    pub description: &'static str,
    pub capabilities: &'static [Capability],
}

/// Shared input available to every render adapter.
#[derive(Debug, Clone, Copy)]
pub struct AdapterContext<'a, 'plan> {
    pub plan: &'a ResolvedPlan<'plan>,
    pub target_host: Option<&'a str>,
}

impl<'a, 'plan> AdapterContext<'a, 'plan> {
    pub const fn new(plan: &'a ResolvedPlan<'plan>) -> Self {
        Self {
            plan,
            target_host: None,
        }
    }

    pub const fn for_host(mut self, host: &'a str) -> Self {
        self.target_host = Some(host);
        self
    }
}

/// Result of rendering one adapter.
#[derive(Debug, Clone)]
pub struct AdapterOutput {
    pub adapter: AdapterId,
    pub root: PathBuf,
    pub target_host: Option<String>,
    pub artifacts: Vec<Artifact>,
}

/// Public interface implemented by built-in and downstream Rust adapters.
pub trait Adapter: Send + Sync {
    fn metadata(&self) -> AdapterMetadata;

    /// Validate adapter-specific requirements without rendering artifacts.
    fn validate(&self, context: &AdapterContext<'_, '_>) -> Result<()>;

    /// Render artifacts for this adapter.
    fn render(&self, context: &AdapterContext<'_, '_>) -> Result<AdapterOutput>;
}
