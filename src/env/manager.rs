use anyhow::{Context, Result};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

pub struct EnvManager {
    vars: HashMap<String, String>,
}

impl EnvManager {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        if !path.as_ref().exists() {
            return Ok(());
        }

        let content = fs::read_to_string(path).context("Failed to read env file")?;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_string();
                let value = value
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                self.vars.insert(key, value);
            }
        }

        Ok(())
    }

    pub fn load_from_map(&mut self, env_vars: &HashMap<String, String>) {
        for (key, value) in env_vars {
            self.vars.insert(key.clone(), value.clone());
        }
    }

    pub fn apply(&self) {
        for (key, value) in &self.vars {
            env::set_var(key, value);
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.vars.get(key)
    }

    pub fn set(&mut self, key: String, value: String) {
        self.vars.insert(key, value);
    }

    pub fn vars(&self) -> &HashMap<String, String> {
        &self.vars
    }
}

impl Default for EnvManager {
    fn default() -> Self {
        Self::new()
    }
}
