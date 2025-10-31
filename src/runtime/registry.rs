use dirs::home_dir;
use include_dir::{include_dir, Dir};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use super::declarative::DeclarativeProvider;
use super::provider::RuntimeProvider;
use crate::errors::{RealmError, Result};

static BUILTIN_RUNTIMES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/runtimes");

/// Registry for runtime providers
pub struct RuntimeRegistry {
  providers: HashMap<String, Arc<dyn RuntimeProvider>>,
}

impl RuntimeRegistry {
  pub fn new() -> Self {
    Self {
      providers: HashMap::new(),
    }
  }

  /// Register a runtime provider
  pub fn register(&mut self, provider: Arc<dyn RuntimeProvider>) {
    let name = provider.name().to_string();
    self.providers.insert(name.clone(), provider.clone());

    // Register aliases
    for alias in provider.aliases() {
      self.providers.insert(alias.to_string(), provider.clone());
    }
  }

  /// Get a runtime provider by name or alias
  pub fn get(&self, name: &str) -> Option<Arc<dyn RuntimeProvider>> {
    self.providers.get(name).cloned()
  }

  /// List all registered runtime names (primary names only, not aliases)
  pub fn list_runtimes(&self) -> Vec<String> {
    let mut names: Vec<String> = self
      .providers
      .iter()
      .filter(|(k, v)| k.as_str() == v.name())
      .map(|(k, _)| k.clone())
      .collect();
    names.sort();
    names
  }

  /// Load runtime providers from builtin and user runtime directories
  pub async fn discover_runtimes(&mut self) -> Result<()> {
    // Load built-in runtimes from embedded directory
    self.load_builtin_runtimes().await?;

    // Load user-defined runtimes from ~/.realm/runtimes-config
    self.load_user_runtimes().await?;

    Ok(())
  }

  async fn load_builtin_runtimes(&mut self) -> Result<()> {
    for entry in BUILTIN_RUNTIMES.entries() {
      if let Some(file) = entry.as_file() {
        let path = file.path();
        let ext = path.extension().and_then(|s| s.to_str());

        if ext == Some("yaml") || ext == Some("yml") {
          match DeclarativeProvider::from_yaml_str(file.contents_utf8().unwrap_or_default()) {
            Ok(provider) => {
              self.register(Arc::new(provider));
            }
            Err(e) => {
              eprintln!("⚠ Failed to load builtin runtime from {:?}: {}", path, e);
            }
          }
        }
      }
    }

    Ok(())
  }

  async fn load_user_runtimes(&mut self) -> Result<()> {
    let runtimes_dir = self.get_runtimes_dir()?;

    if !runtimes_dir.exists() {
      return Ok(());
    }

    let entries = std::fs::read_dir(&runtimes_dir).map_err(|e| {
      RealmError::ValidationError(format!("Failed to read runtimes directory: {}", e))
    })?;

    for entry in entries {
      let entry = entry.map_err(|e| {
        RealmError::ValidationError(format!("Failed to read directory entry: {}", e))
      })?;

      let path = entry.path();
      let ext = path.extension().and_then(|s| s.to_str());
      if path.is_file() && (ext == Some("yaml") || ext == Some("yml")) {
        match DeclarativeProvider::from_file(&path) {
          Ok(provider) => {
            self.register(Arc::new(provider));
          }
          Err(e) => {
            eprintln!("⚠ Failed to load runtime from {:?}: {}", path, e);
          }
        }
      }
    }

    Ok(())
  }

  fn get_runtimes_dir(&self) -> Result<PathBuf> {
    let home = home_dir()
      .ok_or_else(|| RealmError::ValidationError("Could not find home directory".to_string()))?;

    Ok(home.join(".realm").join("runtimes-config"))
  }
}

impl Default for RuntimeRegistry {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::runtime::provider::{PlatformInfo, RuntimeArtifact, RuntimeProvider};
  use async_trait::async_trait;
  use std::path::Path;

  struct TestProvider {
    name: String,
  }

  #[async_trait]
  impl RuntimeProvider for TestProvider {
    fn name(&self) -> &str {
      &self.name
    }

    async fn list_versions(&self) -> Result<Vec<String>> {
      Ok(vec!["1.0.0".to_string()])
    }

    async fn get_artifact(
      &self,
      _version: &str,
      _platform: &PlatformInfo,
    ) -> Result<RuntimeArtifact> {
      unimplemented!()
    }

    async fn install_artifact(
      &self,
      _artifact_data: &[u8],
      _artifact: &RuntimeArtifact,
      _install_dir: &Path,
    ) -> Result<()> {
      unimplemented!()
    }
  }

  #[test]
  fn test_registry() {
    let mut registry = RuntimeRegistry::new();

    let provider = Arc::new(TestProvider {
      name: "test".to_string(),
    });

    registry.register(provider.clone());

    assert!(registry.get("test").is_some());
    assert_eq!(registry.list_runtimes().len(), 1);
  }
}
