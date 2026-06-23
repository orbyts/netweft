//! Observation interfaces for future discovery and drift detection.
//!
//! Version 0.2 defines the boundary only. No observer is run implicitly, so
//! existing deterministic configuration behavior remains unchanged.

use std::collections::BTreeMap;
use std::net::IpAddr;

use anyhow::Result;

/// Facts observed from the current environment.
#[derive(Debug, Default, Clone)]
pub struct ObservationSet {
    pub hosts: BTreeMap<String, HostObservation>,
    pub metadata: BTreeMap<String, String>,
}

impl ObservationSet {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn merge(&mut self, other: Self) {
        self.hosts.extend(other.hosts);
        self.metadata.extend(other.metadata);
    }
}

/// Observed state for one known or candidate host.
#[derive(Debug, Default, Clone)]
pub struct HostObservation {
    pub addresses: Vec<IpAddr>,
    pub interfaces: BTreeMap<String, InterfaceObservation>,
    pub attributes: BTreeMap<String, String>,
}

/// Observed state for one interface.
#[derive(Debug, Default, Clone)]
pub struct InterfaceObservation {
    pub addresses: Vec<IpAddr>,
    pub mac_address: Option<String>,
    pub default_route: bool,
}

/// A provider that observes state without changing it.
pub trait ObservationProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn observe(&self) -> Result<ObservationSet>;
}
