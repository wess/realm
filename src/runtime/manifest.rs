use crate::errors::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Runtime manifest - can be defined in TOML for declarative runtimes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeManifest {
  /// Runtime metadata
  pub runtime: RuntimeMeta,

  /// Version discovery configuration
  pub versions: VersionDiscovery,

  /// Download configuration per platform
  pub downloads: HashMap<String, DownloadConfig>,

  /// Installation instructions
  pub install: InstallConfig,

  /// Optional: Environment configuration
  #[serde(default)]
  pub environment: EnvironmentConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeMeta {
  /// Unique identifier (e.g., "deno", "go", "ruby")
  pub name: String,

  /// Display name
  #[serde(default)]
  pub display_name: Option<String>,

  /// Aliases (e.g., ["py", "python3"])
  #[serde(default)]
  pub aliases: Vec<String>,

  /// Version command to check system installation
  #[serde(default = "default_version_command")]
  pub version_command: String,

  /// Description
  #[serde(default)]
  pub description: Option<String>,

  /// URL where versions can be found
  /// Can be GitHub releases, JSON API, or HTML page
  pub versions_url: String,
}

fn default_version_command() -> String {
  "--version".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum VersionDiscovery {
  /// Fetch from GitHub releases
  #[serde(rename = "github")]
  GitHub {
    repo: String, // e.g., "denoland/deno"
    /// Extract version from tag (regex pattern, first capture group)
    tag_pattern: Option<String>,
    /// Filter tags (e.g., only include stable releases)
    filter: Option<String>,
  },

  /// Fetch from a JSON API
  #[serde(rename = "api")]
  Api {
    url: String,
    /// JSONPath to extract versions
    json_path: String,
  },

  /// Fetch from HTML page (scraping)
  #[serde(rename = "html")]
  Html {
    url: String,
    /// CSS selector or regex pattern
    selector: String,
  },

  /// Static list (for testing or simple runtimes)
  #[serde(rename = "static")]
  Static { versions: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadConfig {
  /// URL template with variables: {version}, {os}, {arch}
  pub url_template: String,

  /// Archive format: "tar.gz", "zip", "binary"
  pub format: String,

  /// Optional: Checksum URL template
  #[serde(default)]
  pub checksum_url: Option<String>,

  /// Optional: Checksum algorithm
  #[serde(default)]
  pub checksum_algo: Option<String>,

  /// OS mapping (realm os -> runtime os naming)
  #[serde(default)]
  pub os_map: HashMap<String, String>,

  /// Arch mapping (realm arch -> runtime arch naming)
  #[serde(default)]
  pub arch_map: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallConfig {
  /// Where the binary is located in the archive (relative path)
  pub binary_path: String,

  /// Additional binaries to symlink
  #[serde(default)]
  pub additional_binaries: Vec<String>,

  /// Strip N leading components from archive paths
  #[serde(default)]
  pub strip_components: u32,

  /// Post-install commands to run
  #[serde(default)]
  pub post_install_commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnvironmentConfig {
  /// Environment variables to set (key-value pairs)
  #[serde(default)]
  pub vars: HashMap<String, String>,

  /// Whether this runtime requires isolation (like Python)
  #[serde(default)]
  pub requires_isolation: bool,

  /// Isolation setup commands
  #[serde(default)]
  pub isolation_commands: Vec<String>,
}

impl RuntimeManifest {
  /// Load manifest from YAML file
  pub fn from_file(path: &PathBuf) -> Result<Self> {
    let content = std::fs::read_to_string(path).map_err(|e| {
      crate::errors::RealmError::ValidationError(format!("Failed to read manifest: {}", e))
    })?;

    serde_yaml::from_str(&content).map_err(|e| {
      crate::errors::RealmError::ValidationError(format!("Failed to parse manifest: {}", e))
    })
  }

  /// Load manifest from YAML string
  pub fn from_yaml_str(yaml: &str) -> Result<Self> {
    serde_yaml::from_str(yaml).map_err(|e| {
      crate::errors::RealmError::ValidationError(format!("Failed to parse manifest: {}", e))
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_deno_manifest() {
    let yaml = r#"
runtime:
  name: deno
  display_name: Deno
  aliases:
    - deno
  description: A secure runtime for JavaScript and TypeScript
  versions_url: "https://github.com/denoland/deno/releases"

versions:
  type: github
  repo: denoland/deno
  tag_pattern: "^v(.+)$"

downloads:
  darwin-arm64:
    url_template: "https://github.com/denoland/deno/releases/download/v{version}/deno-aarch64-apple-darwin.zip"
    format: zip
  darwin-x64:
    url_template: "https://github.com/denoland/deno/releases/download/v{version}/deno-x86_64-apple-darwin.zip"
    format: zip
  linux-x64:
    url_template: "https://github.com/denoland/deno/releases/download/v{version}/deno-x86_64-unknown-linux-gnu.zip"
    format: zip

install:
  binary_path: deno
  strip_components: 0
    "#;

    let manifest = RuntimeManifest::from_yaml_str(yaml).unwrap();
    assert_eq!(manifest.runtime.name, "deno");
    assert_eq!(
      manifest.runtime.versions_url,
      "https://github.com/denoland/deno/releases"
    );
    assert_eq!(manifest.downloads.len(), 3);
  }
}
