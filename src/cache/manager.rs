use crate::errors::{RealmError, RuntimeError, Result};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_TTL_SECONDS: u64 = 86400; // 24 hours

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedData<T> {
  pub timestamp: u64,
  pub data: T,
}

impl<T> CachedData<T> {
  pub fn new(data: T) -> Self {
    let timestamp = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_secs();

    Self { timestamp, data }
  }

  pub fn is_expired(&self, ttl_seconds: u64) -> bool {
    let now = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_secs();

    now - self.timestamp > ttl_seconds
  }
}

pub struct CacheManager {
  cache_dir: PathBuf,
  ttl_seconds: u64,
}

impl CacheManager {
  pub fn new() -> Result<Self> {
    let home = home_dir().ok_or_else(|| {
      RealmError::RuntimeError(RuntimeError::UnsupportedPlatform(
        "Could not find home directory".to_string(),
      ))
    })?;

    let cache_dir = home.join(".realm").join("cache");

    if !cache_dir.exists() {
      fs::create_dir_all(&cache_dir).map_err(|e| {
        RealmError::RuntimeError(RuntimeError::InstallationFailed(format!(
          "Failed to create cache directory: {e}"
        )))
      })?;
    }

    Ok(Self {
      cache_dir,
      ttl_seconds: DEFAULT_TTL_SECONDS,
    })
  }

  pub fn with_ttl(mut self, ttl_seconds: u64) -> Self {
    self.ttl_seconds = ttl_seconds;
    self
  }

  fn get_cache_path(&self, key: &str) -> PathBuf {
    self.cache_dir.join(format!("{}.json", key))
  }

  pub fn get<T>(&self, key: &str) -> Result<Option<T>>
  where
    T: for<'de> Deserialize<'de>,
  {
    let path = self.get_cache_path(key);

    if !path.exists() {
      return Ok(None);
    }

    let contents = fs::read_to_string(&path).map_err(|e| {
      RealmError::RuntimeError(RuntimeError::DownloadFailed(format!(
        "Failed to read cache file: {e}"
      )))
    })?;

    let cached: CachedData<T> = serde_json::from_str(&contents).map_err(|e| {
      RealmError::RuntimeError(RuntimeError::DownloadFailed(format!(
        "Failed to parse cache file: {e}"
      )))
    })?;

    if cached.is_expired(self.ttl_seconds) {
      return Ok(None);
    }

    Ok(Some(cached.data))
  }

  pub fn get_stale<T>(&self, key: &str) -> Result<Option<T>>
  where
    T: for<'de> Deserialize<'de>,
  {
    let path = self.get_cache_path(key);

    if !path.exists() {
      return Ok(None);
    }

    let contents = fs::read_to_string(&path).map_err(|e| {
      RealmError::RuntimeError(RuntimeError::DownloadFailed(format!(
        "Failed to read cache file: {e}"
      )))
    })?;

    let cached: CachedData<T> = serde_json::from_str(&contents).map_err(|e| {
      RealmError::RuntimeError(RuntimeError::DownloadFailed(format!(
        "Failed to parse cache file: {e}"
      )))
    })?;

    Ok(Some(cached.data))
  }

  pub fn set<T>(&self, key: &str, data: T) -> Result<()>
  where
    T: Serialize,
  {
    let cached = CachedData::new(data);
    let json = serde_json::to_string_pretty(&cached).map_err(|e| {
      RealmError::RuntimeError(RuntimeError::DownloadFailed(format!(
        "Failed to serialize cache data: {e}"
      )))
    })?;

    let path = self.get_cache_path(key);
    fs::write(&path, json).map_err(|e| {
      RealmError::RuntimeError(RuntimeError::DownloadFailed(format!(
        "Failed to write cache file: {e}"
      )))
    })?;

    Ok(())
  }

  pub fn clear(&self, key: &str) -> Result<()> {
    let path = self.get_cache_path(key);

    if path.exists() {
      fs::remove_file(&path).map_err(|e| {
        RealmError::RuntimeError(RuntimeError::DownloadFailed(format!(
          "Failed to remove cache file: {e}"
        )))
      })?;
    }

    Ok(())
  }

  pub fn clear_all(&self) -> Result<()> {
    if self.cache_dir.exists() {
      for entry in fs::read_dir(&self.cache_dir).map_err(|e| {
        RealmError::RuntimeError(RuntimeError::DownloadFailed(format!(
          "Failed to read cache directory: {e}"
        )))
      })? {
        let entry = entry.map_err(|e| {
          RealmError::RuntimeError(RuntimeError::DownloadFailed(format!(
            "Failed to read directory entry: {e}"
          )))
        })?;

        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
          fs::remove_file(&path).map_err(|e| {
            RealmError::RuntimeError(RuntimeError::DownloadFailed(format!(
              "Failed to remove cache file: {e}"
            )))
          })?;
        }
      }
    }

    Ok(())
  }

  pub fn cache_dir(&self) -> &Path {
    &self.cache_dir
  }
}

impl Default for CacheManager {
  fn default() -> Self {
    Self::new().expect("Failed to create CacheManager")
  }
}
