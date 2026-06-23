use std::collections::BTreeMap;

use anyhow::{Result, bail};

use super::{Adapter, AdapterId};

/// Registry of adapters compiled into a Netweft application.
#[derive(Default)]
pub struct AdapterRegistry {
    adapters: BTreeMap<AdapterId, Box<dyn Adapter>>,
}

impl AdapterRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<A>(&mut self, adapter: A) -> Result<()>
    where
        A: Adapter + 'static,
    {
        let id = adapter.metadata().id;
        if self.adapters.insert(id, Box::new(adapter)).is_some() {
            bail!("adapter '{id}' is already registered");
        }
        Ok(())
    }

    pub fn get(&self, id: &str) -> Result<&dyn Adapter> {
        self.adapters
            .iter()
            .find_map(|(candidate, adapter)| (candidate.as_str() == id).then_some(adapter.as_ref()))
            .ok_or_else(|| anyhow::anyhow!("unknown adapter '{id}'"))
    }

    pub fn iter(&self) -> impl Iterator<Item = &dyn Adapter> {
        self.adapters.values().map(Box::as_ref)
    }
}
