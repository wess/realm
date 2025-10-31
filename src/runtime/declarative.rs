use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use std::path::{Path, PathBuf};

use super::manifest::{RuntimeManifest, VersionDiscovery};
use super::provider::{ArtifactFormat, PlatformInfo, RuntimeArtifact, RuntimeProvider};
use crate::errors::{RealmError, Result, RuntimeError};

/// A declarative runtime provider that uses a manifest
pub struct DeclarativeProvider {
  manifest: RuntimeManifest,
  client: Client,
}

impl DeclarativeProvider {
  pub fn new(manifest: RuntimeManifest) -> Self {
    Self {
      manifest,
      client: Client::new(),
    }
  }

  pub fn from_file(path: &PathBuf) -> Result<Self> {
    let manifest = RuntimeManifest::from_file(path)?;
    Ok(Self::new(manifest))
  }

  pub fn from_yaml_str(yaml: &str) -> Result<Self> {
    let manifest = RuntimeManifest::from_yaml_str(yaml)?;
    Ok(Self::new(manifest))
  }

  fn get_platform_key(platform: &PlatformInfo) -> String {
    format!("{}-{}", platform.os, platform.arch)
  }

  fn apply_template(&self, template: &str, version: &str, platform: &PlatformInfo) -> String {
    template
      .replace("{version}", version)
      .replace("{os}", &platform.os)
      .replace("{arch}", &platform.arch)
  }
}

#[async_trait]
impl RuntimeProvider for DeclarativeProvider {
  fn name(&self) -> &str {
    &self.manifest.runtime.name
  }

  fn aliases(&self) -> Vec<&str> {
    self
      .manifest
      .runtime
      .aliases
      .iter()
      .map(|s| s.as_str())
      .collect()
  }

  fn display_name(&self) -> &str {
    self
      .manifest
      .runtime
      .display_name
      .as_deref()
      .unwrap_or(self.name())
  }

  async fn list_versions(&self) -> Result<Vec<String>> {
    match &self.manifest.versions {
      VersionDiscovery::GitHub {
        repo, tag_pattern, ..
      } => {
        let url = format!("https://api.github.com/repos/{}/releases", repo);

        let response = self
          .client
          .get(&url)
          .header("User-Agent", "realm")
          .send()
          .await
          .map_err(|e| RealmError::RuntimeError(RuntimeError::VersionFetchFailed(e.to_string())))?;

        let releases: Vec<Value> = response
          .json()
          .await
          .map_err(|e| RealmError::RuntimeError(RuntimeError::VersionFetchFailed(e.to_string())))?;

        let mut versions = Vec::new();
        for release in releases {
          if let Some(tag) = release["tag_name"].as_str() {
            let version = if let Some(pattern) = tag_pattern {
              // Extract version using regex pattern
              if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(caps) = re.captures(tag) {
                  caps
                    .get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_else(|| tag.to_string())
                } else {
                  continue;
                }
              } else {
                tag.to_string()
              }
            } else {
              tag.to_string()
            };
            versions.push(version);
          }
        }

        Ok(versions)
      }

      VersionDiscovery::Api { url, json_path } => {
        let response =
          self.client.get(url).send().await.map_err(|e| {
            RealmError::RuntimeError(RuntimeError::VersionFetchFailed(e.to_string()))
          })?;

        let data: Value = response
          .json()
          .await
          .map_err(|e| RealmError::RuntimeError(RuntimeError::VersionFetchFailed(e.to_string())))?;

        // Simple JSONPath implementation (just handle basic paths)
        let versions = self.extract_versions_from_json(&data, json_path)?;
        Ok(versions)
      }

      VersionDiscovery::Static { versions } => Ok(versions.clone()),

      VersionDiscovery::Html { .. } => {
        // HTML scraping would require additional dependencies (scraper, select, etc.)
        Err(RealmError::RuntimeError(RuntimeError::VersionFetchFailed(
          "HTML version discovery not yet implemented".to_string(),
        )))
      }
    }
  }

  async fn get_artifact(&self, version: &str, platform: &PlatformInfo) -> Result<RuntimeArtifact> {
    let platform_key = Self::get_platform_key(platform);

    let download_config = self.manifest.downloads.get(&platform_key).ok_or_else(|| {
      RealmError::RuntimeError(RuntimeError::UnsupportedPlatform(format!(
        "No download configuration for platform: {}",
        platform_key
      )))
    })?;

    let url = self.apply_template(&download_config.url_template, version, platform);

    let format = match download_config.format.as_str() {
      "tar.gz" | "tgz" => ArtifactFormat::TarGz,
      "zip" => ArtifactFormat::Zip,
      "binary" => ArtifactFormat::Binary,
      _ => {
        return Err(RealmError::RuntimeError(RuntimeError::DownloadFailed(
          format!("Unknown format: {}", download_config.format),
        )))
      }
    };

    let (checksum, checksum_algo) =
      if let Some(checksum_url_template) = &download_config.checksum_url {
        let checksum_url = self.apply_template(checksum_url_template, version, platform);
        let checksum_text = match self.client.get(&checksum_url).send().await {
          Ok(response) => match response.text().await {
            Ok(text) => text
              .lines()
              .next()
              .map(|s| s.split_whitespace().next().unwrap_or("").to_string()),
            Err(_) => None,
          },
          Err(_) => None,
        };

        (checksum_text, download_config.checksum_algo.clone())
      } else {
        (None, None)
      };

    Ok(RuntimeArtifact {
      url,
      checksum,
      checksum_algo,
      format,
    })
  }

  async fn install_artifact(
    &self,
    artifact_data: &[u8],
    artifact: &RuntimeArtifact,
    install_dir: &Path,
  ) -> Result<()> {
    std::fs::create_dir_all(install_dir)
      .map_err(|e| RealmError::RuntimeError(RuntimeError::InstallationFailed(e.to_string())))?;

    match artifact.format {
      ArtifactFormat::TarGz => {
        use flate2::read::GzDecoder;
        use tar::Archive;

        let tar = GzDecoder::new(artifact_data);
        let mut archive = Archive::new(tar);

        archive
          .unpack(install_dir)
          .map_err(|e| RealmError::RuntimeError(RuntimeError::ExtractionFailed(e.to_string())))?;
      }

      ArtifactFormat::Zip => {
        // Use the existing extract_zip_safely function
        super::manager::extract_zip_safely(artifact_data, install_dir)?;
      }

      ArtifactFormat::Binary => {
        let bin_dir = install_dir.join("bin");
        std::fs::create_dir_all(&bin_dir)
          .map_err(|e| RealmError::RuntimeError(RuntimeError::InstallationFailed(e.to_string())))?;

        let binary_path = bin_dir.join(self.name());
        std::fs::write(&binary_path, artifact_data)
          .map_err(|e| RealmError::RuntimeError(RuntimeError::InstallationFailed(e.to_string())))?;

        super::manager::set_executable_permissions(&binary_path)?;
      }
    }

    Ok(())
  }

  async fn post_install(&self, install_dir: &Path) -> Result<()> {
    // Run post-install commands
    for command in &self.manifest.install.post_install_commands {
      let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(install_dir)
        .output()
        .map_err(|e| {
          RealmError::RuntimeError(RuntimeError::InstallationFailed(format!(
            "Failed to run post-install command: {}",
            e
          )))
        })?;

      if !output.status.success() {
        return Err(RealmError::RuntimeError(RuntimeError::InstallationFailed(
          format!(
            "Post-install command failed: {}",
            String::from_utf8_lossy(&output.stderr)
          ),
        )));
      }
    }

    Ok(())
  }

  fn executable_path(&self, install_dir: &Path) -> PathBuf {
    install_dir.join(&self.manifest.install.binary_path)
  }

  fn additional_executables(&self) -> Vec<&str> {
    self
      .manifest
      .install
      .additional_binaries
      .iter()
      .map(|s| s.as_str())
      .collect()
  }

  fn environment_vars(&self, install_dir: &PathBuf) -> Vec<(String, String)> {
    self
      .manifest
      .environment
      .vars
      .iter()
      .map(|(k, v)| {
        let value = v.replace("{install_dir}", &install_dir.to_string_lossy());
        (k.clone(), value)
      })
      .collect()
  }

  fn requires_isolation(&self) -> bool {
    self.manifest.environment.requires_isolation
  }
}

impl DeclarativeProvider {
  fn extract_versions_from_json(&self, data: &Value, path: &str) -> Result<Vec<String>> {
    // Simple JSONPath implementation for basic paths like "versions" or "data.versions"
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = data;

    for part in parts {
      current = current.get(part).ok_or_else(|| {
        RealmError::RuntimeError(RuntimeError::VersionFetchFailed(format!(
          "Path not found: {}",
          part
        )))
      })?;
    }

    if let Some(array) = current.as_array() {
      let versions: Vec<String> = array
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
      Ok(versions)
    } else {
      Err(RealmError::RuntimeError(RuntimeError::VersionFetchFailed(
        "Expected array at path".to_string(),
      )))
    }
  }
}
