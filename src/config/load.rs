use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::de::DeserializeOwned;

use crate::model::{ConfigBundle, SettingsFile};

pub struct ConfigLoader<'a> {
    config_dir: &'a Path,
}

impl<'a> ConfigLoader<'a> {
    pub fn new(config_dir: &'a Path) -> Self {
        Self { config_dir }
    }

    pub fn load(&self, location_override: Option<&str>) -> Result<ConfigBundle> {
        let settings: SettingsFile = self.read("netweft.toml")?;
        let location_name = location_override.unwrap_or(&settings.active_location);

        let location_path = PathBuf::from("locations").join(format!("{location_name}.toml"));

        Ok(ConfigBundle {
            settings,
            inventory: self.read("inventory.toml")?,
            networks: self.read("networks.toml")?,
            services: self.read("services.toml")?,
            dns: self.read("dns.toml")?,
            allocations: self.read("allocations.toml")?,
            location: self.read_path(&location_path)?,
        })
    }

    fn read<T: DeserializeOwned>(&self, relative: &str) -> Result<T> {
        self.read_path(Path::new(relative))
    }

    fn read_path<T: DeserializeOwned>(&self, relative: &Path) -> Result<T> {
        let path = self.config_dir.join(relative);
        let source = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;

        toml::from_str(&source).with_context(|| format!("failed to deserialize {}", path.display()))
    }
}
