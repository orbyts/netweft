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
            docker: self.read_optional("docker.toml")?,
            ssh: self.read_optional("ssh.toml")?,
            cloudflare: self.read_optional("cloudflare.toml")?,
            services: self.read("services.toml")?,
            guests: self.read_optional("guests.toml")?,
            mounts: self.read_optional("mounts.toml")?,
            nas_permissions: self.read_optional("nas-permissions.toml")?,
            proxmox_storages: self.read_optional("proxmox-storages.toml")?,
            proxmox_sdn: self.read_optional("proxmox-sdn.toml")?,
            dns: self.read("dns.toml")?,
            allocations: self.read("allocations.toml")?,
            location: self.read_path(&location_path)?,
        })
    }

    fn read<T: DeserializeOwned>(&self, relative: &str) -> Result<T> {
        self.read_path(Path::new(relative))
    }

    fn read_optional<T>(&self, relative: &str) -> Result<T>
    where
        T: DeserializeOwned + Default,
    {
        let path = self.config_dir.join(relative);
        if !path.exists() {
            return Ok(T::default());
        }
        self.read(relative)
    }

    fn read_path<T: DeserializeOwned>(&self, relative: &Path) -> Result<T> {
        let path = self.config_dir.join(relative);
        let source = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;

        toml::from_str(&source).with_context(|| format!("failed to deserialize {}", path.display()))
    }
}
