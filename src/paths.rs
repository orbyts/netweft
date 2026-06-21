use std::env;
use std::fmt;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

#[derive(Debug, Clone)]
pub struct NetweftPaths {
    pub config_dir: PathBuf,
    pub generated_dir: PathBuf,
    pub state_dir: PathBuf,
    pub cache_dir: PathBuf,
}

impl NetweftPaths {
    pub fn resolve(cli_config_dir: Option<&Path>) -> Result<Self> {
        let home = home_dir()?;

        let config_dir = if let Some(path) = cli_config_dir {
            path.to_path_buf()
        } else if let Some(path) = env::var_os("NETWEFT_CONFIG_DIR") {
            PathBuf::from(path)
        } else if let Some(path) = env::var_os("XDG_CONFIG_HOME") {
            PathBuf::from(path).join("netweft")
        } else {
            home.join(".config/netweft")
        };

        let generated_dir = env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".local/share"))
            .join("netweft/generated");

        let state_dir = env::var_os("XDG_STATE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".local/state"))
            .join("netweft");

        let cache_dir = env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join(".cache"))
            .join("netweft");

        Ok(Self {
            config_dir,
            generated_dir,
            state_dir,
            cache_dir,
        })
    }
}

impl fmt::Display for NetweftPaths {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Config:    {}", self.config_dir.display())?;
        writeln!(f, "Generated: {}", self.generated_dir.display())?;
        writeln!(f, "State:     {}", self.state_dir.display())?;
        write!(f, "Cache:     {}", self.cache_dir.display())
    }
}

fn home_dir() -> Result<PathBuf> {
    let Some(home) = env::var_os("HOME") else {
        bail!("HOME is not set and Netweft cannot resolve its default paths");
    };

    let path = PathBuf::from(home);

    if path.as_os_str().is_empty() {
        bail!("HOME is empty");
    }

    Ok(path)
}
