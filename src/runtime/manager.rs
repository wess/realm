use anyhow::{anyhow, Context, Result};
use dirs::home_dir;
use flate2::read::GzDecoder;
use reqwest::Client;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tar::Archive;

use super::types::Runtime;

pub struct RuntimeManager {
  realm_dir: PathBuf,
  http_client: Client,
}

impl RuntimeManager {
  pub fn new() -> Result<Self> {
    let home = home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
    let realm_dir = home.join(".realm");

    if !realm_dir.exists() {
      fs::create_dir_all(&realm_dir).context("Failed to create .realm directory")?;
    }

    let user_agent: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

    let http_client = Client::builder()
      .user_agent(user_agent)
      .build()
      .context("Failed to initialize HTTP client")?;

    Ok(Self {
      realm_dir,
      http_client,
    })
  }

  pub fn get_runtime_versions_dir(&self, runtime: &Runtime) -> PathBuf {
    self.realm_dir.join(runtime.name())
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

    let os = std::env::consts::OS;
    let arch = match std::env::consts::ARCH {
      "x86_64" => "x64",
      "aarch64" => "aarch64",
      _ => {
        return Err(anyhow!(
          "Unsupported architecture: {}",
          std::env::consts::ARCH
        ))
      }
    };

    let download_url = format!(
      "https://github.com/oven-sh/bun/releases/download/bun-v{actual_version}/bun-{os}-{arch}.zip"
    );

    let response = self
      .http_client
      .get(&download_url)
      .send()
      .await
      .context("Failed to download Bun")?;

    if !response.status().is_success() {
      return Err(anyhow!(
        "Failed to download Bun: HTTP {}",
        response.status()
      ));
    }

    let bytes = response.bytes().await.context("Failed to read download")?;

    let version_dir = self
      .get_runtime_versions_dir(&Runtime::Bun(actual_version.clone()))
      .join(&actual_version);
    fs::create_dir_all(&version_dir).context("Failed to create version directory")?;

    let temp_file = version_dir.join("bun.zip");
    fs::write(&temp_file, bytes).context("Failed to write temp file")?;

    let output = Command::new("unzip")
      .arg("-o")
      .arg(&temp_file)
      .arg("-d")
      .arg(&version_dir)
      .output()
      .context("Failed to extract Bun")?;

    if !output.status.success() {
      return Err(anyhow!(
        "Failed to extract Bun: {}",
        String::from_utf8_lossy(&output.stderr)
      ));
    }

    let extracted_dir = version_dir.join(format!("bun-{os}-{arch}"));
    let extracted_bun = extracted_dir.join("bun");
    let target_bun = version_dir.join("bun");

    if extracted_bun.exists() {
      fs::rename(extracted_bun, target_bun).context("Failed to move bun binary")?;
    }

    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      let mut perms = fs::metadata(version_dir.join("bun"))?.permissions();
      perms.set_mode(0o755);
      fs::set_permissions(version_dir.join("bun"), perms)?;
    }

    let _ = fs::remove_file(temp_file);
    let _ = fs::remove_dir_all(extracted_dir);

    println!("Bun {actual_version} installed successfully");
    Ok(())
  }

  async fn install_node_version(&self, version: &str) -> Result<()> {
    println!("Installing Node.js {version}");

    let actual_version = if version == "latest" {
      self.get_latest_node_version().await?
    } else {
      version.to_string()
    };

    let os = match std::env::consts::OS {
      "macos" => "darwin",
      "linux" => "linux",
      _ => {
        return Err(anyhow!(
          "Unsupported OS for Node.js installation: {}",
          std::env::consts::OS
        ))
      }
    };

    let arch = match std::env::consts::ARCH {
      "x86_64" => "x64",
      "aarch64" => "arm64",
      _ => {
        return Err(anyhow!(
          "Unsupported architecture: {}",
          std::env::consts::ARCH
        ))
      }
    };

    let download_url = format!(
      "https://nodejs.org/dist/v{actual_version}/node-v{actual_version}-{os}-{arch}.tar.gz"
    );

    let response = self
      .http_client
      .get(&download_url)
      .send()
      .await
      .context("Failed to download Node.js")?;

    if !response.status().is_success() {
      return Err(anyhow!(
        "Failed to download Node.js: HTTP {}",
        response.status()
      ));
    }

    let bytes = response.bytes().await.context("Failed to read download")?;

    let version_dir = self
      .get_runtime_versions_dir(&Runtime::Node(actual_version.clone()))
      .join(&actual_version);
    fs::create_dir_all(&version_dir).context("Failed to create version directory")?;

    // Extract tar.gz
    let tar_gz = std::io::Cursor::new(bytes);
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);

    archive
      .unpack(&version_dir)
      .context("Failed to extract Node.js")?;

    // Move extracted contents to proper location
    let extracted_dir = version_dir.join(format!("node-v{actual_version}-{os}-{arch}"));
    if extracted_dir.exists() {
      // Move contents from extracted_dir to version_dir
      for entry in fs::read_dir(&extracted_dir)? {
        let entry = entry?;
        let src = entry.path();
        let dst = version_dir.join(entry.file_name());
        if src.is_dir() {
          Self::copy_dir(&src, &dst)?;
        } else {
          fs::copy(&src, &dst)?;
        }
      }
      fs::remove_dir_all(&extracted_dir)?;
    }

    println!("Node.js {actual_version} installed successfully");
    Ok(())
  }

  fn copy_dir(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
      let entry = entry?;
      let src_path = entry.path();
      let dst_path = dst.join(entry.file_name());
      if src_path.is_dir() {
        Self::copy_dir(&src_path, &dst_path)?;
      } else {
        fs::copy(&src_path, &dst_path)?;
      }
    }
    Ok(())
  }

  async fn get_latest_bun_version(&self) -> Result<String> {
    let response = self
      .http_client
      .get("https://api.github.com/repos/oven-sh/bun/releases/latest")
      .send()
      .await
      .context("Failed to fetch latest Bun version")?;

    let json: serde_json::Value = response
      .json()
      .await
      .context("Failed to parse GitHub API response")?;

    let tag_name = json["tag_name"]
      .as_str()
      .ok_or_else(|| anyhow!("Could not find tag_name in GitHub API response"))?;

    let version = tag_name.strip_prefix("bun-v").unwrap_or(tag_name);

    Ok(version.to_string())
  }

  async fn get_latest_node_version(&self) -> Result<String> {
    let response = self
      .http_client
      .get("https://nodejs.org/dist/index.json")
      .send()
      .await
      .context("Failed to fetch Node.js versions")?;

    let versions: serde_json::Value = response
      .json()
      .await
      .context("Failed to parse Node.js versions response")?;

    if let Some(latest) = versions.as_array().and_then(|arr| arr.first()) {
      if let Some(version_str) = latest["version"].as_str() {
        let version = version_str.strip_prefix("v").unwrap_or(version_str);
        return Ok(version.to_string());
      }
    }

    Err(anyhow!("Could not find latest Node.js version"))
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
      return Err(anyhow!(
        "{} version {} is not installed",
        runtime.name(),
        runtime.version()
      ));
    }

    Command::new(runtime_path)
      .args(args)
      .spawn()
      .context(format!("Failed to start {}", runtime.name()))
  }
}

impl Default for RuntimeManager {
  fn default() -> Self {
    Self::new().expect("Failed to create RuntimeManager")
  }
}
