use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::process::ProcessConfig;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RealmConfig {
    #[serde(default)]
    pub env: HashMap<String, String>,
    pub env_file: Option<String>,
    #[serde(default)]
    pub processes: HashMap<String, ProcessConfig>,
    #[serde(default = "default_proxy_port")]
    pub proxy_port: u16,
}

fn default_proxy_port() -> u16 {
    8000
}

impl Default for RealmConfig {
    fn default() -> Self {
        Self {
            env: HashMap::new(),
            env_file: Some(".env".to_string()),
            processes: HashMap::new(),
            proxy_port: 8000,
        }
    }
}

impl RealmConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path).context("Failed to read realm.yml")?;
        let config: RealmConfig =
            serde_yaml::from_str(&content).context("Failed to parse realm.yml")?;
        Ok(config)
    }

    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        Self::load(path).unwrap_or_default()
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_yaml::to_string(self).context("Failed to serialize config")?;
        fs::write(path, content).context("Failed to write realm.yml")?;
        Ok(())
    }
}
