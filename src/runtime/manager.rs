use crate::errors::{RealmError, RuntimeError, Result};
use dirs::home_dir;
use flate2::read::GzDecoder;
use reqwest::Client;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tar::Archive;

use super::types::Runtime;

pub fn validate_download_url(url: &str, allowed_hosts: &[String]) -> Result<()> {
  let parsed_url = url::Url::parse(url).map_err(|e| {
    RealmError::RuntimeError(RuntimeError::DownloadFailed(format!(
      "Invalid URL: {e}"
    )))
  })?;

  let host = parsed_url.host_str().ok_or_else(|| {
    RealmError::RuntimeError(RuntimeError::DownloadFailed(
      "URL has no host".to_string(),
    ))
  })?;

  if !allowed_hosts.iter().any(|allowed| host.ends_with(allowed)) {
    return Err(RealmError::RuntimeError(RuntimeError::DownloadFailed(
      format!("Host not allowed: {host}"),
    )));
  }

  if parsed_url.scheme() != "https" {
    return Err(RealmError::RuntimeError(RuntimeError::DownloadFailed(
      "Only HTTPS URLs are allowed".to_string(),
    )));
  }

  Ok(())
}

pub fn get_platform_info() -> Result<(String, String)> {
  let os = match std::env::consts::OS {
    "macos" => "darwin",
    "linux" => "linux",
    unsupported => {
      return Err(RealmError::RuntimeError(RuntimeError::UnsupportedPlatform(
        format!("Unsupported OS: {unsupported}"),
      )));
    }
  };

  let arch = match std::env::consts::ARCH {
    "x86_64" => "x64",
    "aarch64" => "arm64",
    unsupported => {
      return Err(RealmError::RuntimeError(RuntimeError::UnsupportedPlatform(
        format!("Unsupported architecture: {unsupported}"),
      )));
    }
  };

  Ok((os.to_string(), arch.to_string()))
}

pub fn extract_zip_safely(zip_bytes: &[u8], extract_to: &Path) -> Result<()> {
  let temp_file = extract_to.join("temp.zip");
  fs::write(&temp_file, zip_bytes).map_err(|e| {
    RealmError::RuntimeError(RuntimeError::ExtractionFailed(format!(
      "Failed to write temp file: {e}"
    )))
  })?;

  let output = Command::new("unzip")
    .arg("-o")
    .arg(&temp_file)
    .arg("-d")
    .arg(extract_to)
    .output()
    .map_err(|e| {
      RealmError::RuntimeError(RuntimeError::ExtractionFailed(format!(
        "Failed to run unzip: {e}"
      )))
    })?;

  if !output.status.success() {
    return Err(RealmError::RuntimeError(RuntimeError::ExtractionFailed(
      format!(
        "Unzip failed: {}",
        String::from_utf8_lossy(&output.stderr)
      ),
    )));
  }

  let _ = fs::remove_file(temp_file);
  Ok(())
}

pub fn set_executable_permissions(path: &Path) -> Result<()> {
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)
      .map_err(|e| {
        RealmError::RuntimeError(RuntimeError::InstallationFailed(format!(
          "Failed to get file metadata: {e}"
        )))
      })?
      .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).map_err(|e| {
      RealmError::RuntimeError(RuntimeError::InstallationFailed(format!(
        "Failed to set permissions: {e}"
      )))
    })?;
  }
  Ok(())
}

pub fn cleanup_temp_directories(temp_dirs: &[PathBuf]) {
  for dir in temp_dirs {
    let _ = fs::remove_dir_all(dir);
  }
}

pub struct RuntimeConfig {
  pub realm_dir: PathBuf,
  pub http_client: Client,
  pub allowed_hosts: Vec<String>,
  pub verify_checksums: bool,
}

pub fn create_runtime_config() -> Result<RuntimeConfig> {
  let home = home_dir().ok_or_else(|| {
    RealmError::RuntimeError(RuntimeError::UnsupportedPlatform(
      "Could not find home directory".to_string(),
    ))
  })?;
  let realm_dir = home.join(".realm");

  if !realm_dir.exists() {
    fs::create_dir_all(&realm_dir).map_err(|e| {
      RealmError::RuntimeError(RuntimeError::InstallationFailed(format!(
        "Failed to create .realm directory: {e}"
      )))
    })?;
  }

  let user_agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

  let http_client = Client::builder()
    .user_agent(user_agent)
    .timeout(std::time::Duration::from_secs(300))
    .build()
    .map_err(|e| {
      RealmError::RuntimeError(RuntimeError::InstallationFailed(format!(
        "Failed to initialize HTTP client: {e}"
      )))
    })?;

  Ok(RuntimeConfig {
    realm_dir,
    http_client,
    allowed_hosts: vec![
      "github.com".to_string(),
      "nodejs.org".to_string(),
      "api.github.com".to_string(),
    ],
    verify_checksums: true,
  })
}

pub struct RuntimeManager {
  config: RuntimeConfig,
}

impl RuntimeManager {
  pub fn new() -> Result<Self> {
    let config = create_runtime_config()?;
    Ok(Self { config })
  }

  pub fn get_runtime_versions_dir(&self, runtime: &Runtime) -> PathBuf {
    self.config.realm_dir.join(runtime.name())
  }

  pub fn get_runtime_path(&self, runtime: &Runtime) -> PathBuf {
    match runtime {
      Runtime::Bun(version) => self
        .get_runtime_versions_dir(runtime)
        .join(version)
        .join("bun"),
      Runtime::Node(version) => self
        .get_runtime_versions_dir(runtime)
        .join(version)
        .join("bin")
        .join("node"),
    }
  }

  pub fn is_version_installed(&self, runtime: &Runtime) -> bool {
    self.get_runtime_path(runtime).exists()
  }

  pub async fn install_version(&self, runtime: &Runtime) -> Result<()> {
    if self.is_version_installed(runtime) {
      return Ok(());
    }

    match runtime {
      Runtime::Bun(version) => self.install_bun_version(version).await,
      Runtime::Node(version) => self.install_node_version(version).await,
    }
  }

  async fn install_bun_version(&self, version: &str) -> Result<()> {
    println!("Installing Bun {version}");

    let actual_version = if version == "latest" {
      self.get_latest_bun_version().await?
    } else {
      version.to_string()
    };

    let (os, arch) = get_platform_info()?;
    let arch = match arch.as_str() {
      "arm64" => "aarch64",
      other => other,
    };

    let download_url = format!(
      "https://github.com/oven-sh/bun/releases/download/bun-v{actual_version}/bun-{os}-{arch}.zip"
    );

    validate_download_url(&download_url, &self.config.allowed_hosts)?;

    let version_dir = self
      .get_runtime_versions_dir(&Runtime::Bun(actual_version.clone()))
      .join(&actual_version);

    // Attempt download with retries
    const MAX_RETRIES: u32 = 3;
    let mut last_error = None;

    for attempt in 1..=MAX_RETRIES {
      if attempt > 1 {
        println!("Retry {}/{MAX_RETRIES}...", attempt - 1);
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
      }

      match self.download_and_install_bun(&download_url, &version_dir, &actual_version, &os, &arch).await {
        Ok(_) => {
          println!("Bun {actual_version} installed successfully");
          return Ok(());
        }
        Err(e) => {
          last_error = Some(e);
          // Clean up partial installation
          let _ = fs::remove_dir_all(&version_dir);
        }
      }
    }

    Err(last_error.unwrap_or_else(|| {
      RealmError::RuntimeError(RuntimeError::DownloadFailed(
        "Unknown error during installation".to_string()
      ))
    }))
  }

  async fn download_and_install_bun(&self, download_url: &str, version_dir: &Path, _actual_version: &str, os: &str, arch: &str) -> Result<()> {
    let response = self
      .config.http_client
      .get(download_url)
      .send()
      .await
      .map_err(|e| {
        RealmError::RuntimeError(RuntimeError::DownloadFailed(format!(
          "Network error: {e}. Check your internet connection and try again."
        )))
      })?;

    if !response.status().is_success() {
      return Err(RealmError::RuntimeError(RuntimeError::DownloadFailed(
        format!("HTTP {} - The requested Bun version may not exist. Visit https://github.com/oven-sh/bun/releases to see available versions.", response.status()),
      )));
    }

    let bytes = response.bytes().await.map_err(|e| {
      RealmError::RuntimeError(RuntimeError::DownloadFailed(format!(
        "Failed to download file: {e}. The connection may have been interrupted."
      )))
    })?;

    fs::create_dir_all(version_dir).map_err(|e| {
      RealmError::RuntimeError(RuntimeError::InstallationFailed(format!(
        "Failed to create directory {}: {e}. Check disk space and permissions.",
        version_dir.display()
      )))
    })?;

    extract_zip_safely(&bytes, version_dir)?;

    let extracted_dir = version_dir.join(format!("bun-{os}-{arch}"));
    let extracted_bun = extracted_dir.join("bun");
    let target_bun = version_dir.join("bun");

    if extracted_bun.exists() {
      fs::rename(extracted_bun, &target_bun).map_err(|e| {
        RealmError::RuntimeError(RuntimeError::InstallationFailed(format!(
          "Failed to move bun binary: {e}"
        )))
      })?;
      set_executable_permissions(&target_bun)?;
    } else {
      return Err(RealmError::RuntimeError(RuntimeError::ExtractionFailed(
        format!("Expected binary not found in archive. The download may be corrupted.")
      )));
    }

    cleanup_temp_directories(&[extracted_dir]);
    Ok(())
  }

  async fn install_node_version(&self, version: &str) -> Result<()> {
    println!("Installing Node.js {version}");

    let actual_version = if version == "latest" {
      self.get_latest_node_version().await?
    } else {
      version.to_string()
    };

    let (os, arch) = get_platform_info()?;

    let download_url = format!(
      "https://nodejs.org/dist/v{actual_version}/node-v{actual_version}-{os}-{arch}.tar.gz"
    );

    validate_download_url(&download_url, &self.config.allowed_hosts)?;

    let version_dir = self
      .get_runtime_versions_dir(&Runtime::Node(actual_version.clone()))
      .join(&actual_version);

    // Attempt download with retries
    const MAX_RETRIES: u32 = 3;
    let mut last_error = None;

    for attempt in 1..=MAX_RETRIES {
      if attempt > 1 {
        println!("Retry {}/{MAX_RETRIES}...", attempt - 1);
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
      }

      match self.download_and_install_node(&download_url, &version_dir, &actual_version, &os, &arch).await {
        Ok(_) => {
          println!("Node.js {actual_version} installed successfully");
          return Ok(());
        }
        Err(e) => {
          last_error = Some(e);
          // Clean up partial installation
          let _ = fs::remove_dir_all(&version_dir);
        }
      }
    }

    Err(last_error.unwrap_or_else(|| {
      RealmError::RuntimeError(RuntimeError::DownloadFailed(
        "Unknown error during installation".to_string()
      ))
    }))
  }

  async fn download_and_install_node(&self, download_url: &str, version_dir: &Path, actual_version: &str, os: &str, arch: &str) -> Result<()> {
    let response = self
      .config.http_client
      .get(download_url)
      .send()
      .await
      .map_err(|e| {
        RealmError::RuntimeError(RuntimeError::DownloadFailed(format!(
          "Network error: {e}. Check your internet connection and try again."
        )))
      })?;

    if !response.status().is_success() {
      return Err(RealmError::RuntimeError(RuntimeError::DownloadFailed(
        format!("HTTP {} - The requested Node.js version may not exist. Visit https://nodejs.org/dist/ to see available versions.", response.status()),
      )));
    }

    let bytes = response.bytes().await.map_err(|e| {
      RealmError::RuntimeError(RuntimeError::DownloadFailed(format!(
        "Failed to download file: {e}. The connection may have been interrupted."
      )))
    })?;

    fs::create_dir_all(version_dir).map_err(|e| {
      RealmError::RuntimeError(RuntimeError::InstallationFailed(format!(
        "Failed to create directory {}: {e}. Check disk space and permissions.",
        version_dir.display()
      )))
    })?;

    // Extract tar.gz
    let tar_gz = std::io::Cursor::new(bytes);
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);

    archive
      .unpack(version_dir)
      .map_err(|e| {
        RealmError::RuntimeError(RuntimeError::ExtractionFailed(format!(
          "Failed to extract archive: {e}. The download may be corrupted."
        )))
      })?;

    // Move extracted contents to proper location
    let extracted_dir = version_dir.join(format!("node-v{actual_version}-{os}-{arch}"));
    if extracted_dir.exists() {
      // Move contents from extracted_dir to version_dir
      for entry in fs::read_dir(&extracted_dir).map_err(|e| {
        RealmError::RuntimeError(RuntimeError::InstallationFailed(format!(
          "Failed to read directory: {e}"
        )))
      })? {
        let entry = entry.map_err(|e| {
          RealmError::RuntimeError(RuntimeError::InstallationFailed(format!(
            "Failed to read directory entry: {e}"
          )))
        })?;
        let src = entry.path();
        let dst = version_dir.join(entry.file_name());
        if src.is_dir() {
          copy_dir(&src, &dst)?;
        } else {
          fs::copy(&src, &dst).map_err(|e| {
            RealmError::RuntimeError(RuntimeError::InstallationFailed(format!(
              "Failed to copy file: {e}"
            )))
          })?;
        }
      }
      cleanup_temp_directories(&[extracted_dir]);
    } else {
      return Err(RealmError::RuntimeError(RuntimeError::ExtractionFailed(
        format!("Expected directory not found in archive. The download may be corrupted.")
      )));
    }

    Ok(())
  }

  async fn get_latest_bun_version(&self) -> Result<String> {
    let url = "https://api.github.com/repos/oven-sh/bun/releases/latest";
    validate_download_url(url, &self.config.allowed_hosts)?;

    let response = self
      .config.http_client
      .get(url)
      .send()
      .await
      .map_err(|e| {
        RealmError::RuntimeError(RuntimeError::DownloadFailed(format!(
          "Request failed: {e}"
        )))
      })?;

    let json: serde_json::Value = response.json().await.map_err(|e| {
      RealmError::RuntimeError(RuntimeError::DownloadFailed(format!(
        "Failed to parse GitHub API response: {e}"
      )))
    })?;

    let tag_name = json["tag_name"].as_str().ok_or_else(|| {
      RealmError::RuntimeError(RuntimeError::DownloadFailed(
        "Could not find tag_name in GitHub API response".to_string(),
      ))
    })?;

    let version = tag_name.strip_prefix("bun-v").unwrap_or(tag_name);
    Ok(version.to_string())
  }

  async fn get_latest_node_version(&self) -> Result<String> {
    let url = "https://nodejs.org/dist/index.json";
    validate_download_url(url, &self.config.allowed_hosts)?;

    let response = self
      .config.http_client
      .get(url)
      .send()
      .await
      .map_err(|e| {
        RealmError::RuntimeError(RuntimeError::DownloadFailed(format!(
          "Request failed: {e}"
        )))
      })?;

    let versions: serde_json::Value = response.json().await.map_err(|e| {
      RealmError::RuntimeError(RuntimeError::DownloadFailed(format!(
        "Failed to parse Node.js versions response: {e}"
      )))
    })?;

    if let Some(latest) = versions.as_array().and_then(|arr| arr.first()) {
      if let Some(version_str) = latest["version"].as_str() {
        let version = version_str.strip_prefix("v").unwrap_or(version_str);
        return Ok(version.to_string());
      }
    }

    Err(RealmError::RuntimeError(RuntimeError::DownloadFailed(
      "Could not find latest Node.js version".to_string(),
    )))
  }

  pub fn get_npm_path(&self, runtime: &Runtime) -> Option<PathBuf> {
    match runtime {
      Runtime::Node(version) => {
        let npm_path = self
          .get_runtime_versions_dir(runtime)
          .join(version)
          .join("bin")
          .join("npm");
        if npm_path.exists() {
          Some(npm_path)
        } else {
          None
        }
      }
      Runtime::Bun(_) => None, // Bun doesn't use npm
    }
  }

  pub fn run_runtime(&self, runtime: &Runtime, args: &[&str]) -> Result<std::process::Child> {
    let runtime_path = self.get_runtime_path(runtime);

    if !runtime_path.exists() {
      return Err(RealmError::RuntimeError(RuntimeError::NotInstalled(
        format!("{} version {}", runtime.name(), runtime.version()),
      )));
    }

    Command::new(runtime_path)
      .args(args)
      .spawn()
      .map_err(|e| {
        RealmError::RuntimeError(RuntimeError::InstallationFailed(format!(
          "Failed to start {}: {e}",
          runtime.name()
        )))
      })
  }
}

fn copy_dir(src: &Path, dst: &Path) -> Result<()> {
  fs::create_dir_all(dst).map_err(|e| {
    RealmError::RuntimeError(RuntimeError::InstallationFailed(format!(
      "Failed to create directory: {e}"
    )))
  })?;
  for entry in fs::read_dir(src).map_err(|e| {
    RealmError::RuntimeError(RuntimeError::InstallationFailed(format!(
      "Failed to read directory: {e}"
    )))
  })? {
    let entry = entry.map_err(|e| {
      RealmError::RuntimeError(RuntimeError::InstallationFailed(format!(
        "Failed to read directory entry: {e}"
      )))
    })?;
    let src_path = entry.path();
    let dst_path = dst.join(entry.file_name());
    if src_path.is_dir() {
      copy_dir(&src_path, &dst_path)?;
    } else {
      fs::copy(&src_path, &dst_path).map_err(|e| {
        RealmError::RuntimeError(RuntimeError::InstallationFailed(format!(
          "Failed to copy file: {e}"
        )))
      })?;
    }
  }
  Ok(())
}

impl Default for RuntimeManager {
  fn default() -> Self {
    Self::new().expect("Failed to create RuntimeManager")
  }
}