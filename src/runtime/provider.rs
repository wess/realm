use crate::errors::Result;
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// Platform information for runtime downloads
#[derive(Debug, Clone)]
pub struct PlatformInfo {
  pub os: String,   // "darwin", "linux", "windows"
  pub arch: String, // "x64", "arm64"
}

/// Represents a downloadable runtime artifact
#[derive(Debug, Clone)]
pub struct RuntimeArtifact {
  pub url: String,
  pub checksum: Option<String>,
  pub checksum_algo: Option<String>, // "sha256", "sha512"
  pub format: ArtifactFormat,
}

#[derive(Debug, Clone)]
pub enum ArtifactFormat {
  TarGz,
  Zip,
  Binary,
}

/// Trait that all runtime providers must implement
#[async_trait]
pub trait RuntimeProvider: Send + Sync {
  /// Unique identifier for this runtime (e.g., "bun", "node", "deno")
  fn name(&self) -> &str;

  /// Aliases for this runtime (e.g., ["py", "python3"] for python)
  fn aliases(&self) -> Vec<&str> {
    vec![]
  }

  /// Display name for UI/logs
  fn display_name(&self) -> &str {
    self.name()
  }

  /// Check if this runtime is available on the system PATH
  async fn is_available_on_system(&self) -> bool {
    if let Ok(output) = std::process::Command::new(self.name())
      .arg("--version")
      .output()
    {
      output.status.success()
    } else {
      false
    }
  }

  /// Get the system-installed version (if available)
  async fn system_version(&self) -> Option<String> {
    None
  }

  /// List all available versions for this runtime
  async fn list_versions(&self) -> Result<Vec<String>>;

  /// Resolve "latest" to an actual version
  async fn resolve_latest(&self) -> Result<String> {
    let versions = self.list_versions().await?;
    versions.first().cloned().ok_or_else(|| {
      crate::errors::RealmError::RuntimeError(crate::errors::RuntimeError::VersionNotFound(
        "No versions available".to_string(),
      ))
    })
  }

  /// Get download URL and metadata for a specific version
  async fn get_artifact(&self, version: &str, platform: &PlatformInfo) -> Result<RuntimeArtifact>;

  /// Install a downloaded artifact to the target directory
  async fn install_artifact(
    &self,
    artifact_data: &[u8],
    artifact: &RuntimeArtifact,
    install_dir: &Path,
  ) -> Result<()>;

  /// Post-installation setup (e.g., creating symlinks, setting permissions)
  async fn post_install(&self, _install_dir: &Path) -> Result<()> {
    Ok(())
  }

  /// Get the path to the main executable after installation
  fn executable_path(&self, install_dir: &Path) -> PathBuf {
    install_dir.join("bin").join(self.name())
  }

  /// Additional executables to symlink (e.g., ["npm", "npx"] for node)
  fn additional_executables(&self) -> Vec<&str> {
    vec![]
  }

  /// Environment variables to set when activating this runtime
  fn environment_vars(&self, _install_dir: &PathBuf) -> Vec<(String, String)> {
    vec![]
  }

  /// Check if this runtime requires special isolation (like Python virtualenv)
  fn requires_isolation(&self) -> bool {
    false
  }

  /// Setup isolation environment (for Python-like runtimes)
  async fn setup_isolation(&self, _venv_dir: &PathBuf, _install_dir: &PathBuf) -> Result<()> {
    Ok(())
  }
}
