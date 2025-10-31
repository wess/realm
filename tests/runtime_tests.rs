use realm::runtime::{Runtime, RuntimeManager};

#[test]
fn test_runtime_parse_bun() {
  let runtime = Runtime::parse("bun").unwrap();
  assert_eq!(runtime.name(), "bun");
  assert_eq!(runtime.version(), "latest");

  let runtime = Runtime::parse("bun@1.0.1").unwrap();
  assert_eq!(runtime.name(), "bun");
  assert_eq!(runtime.version(), "1.0.1");
}

#[test]
fn test_runtime_parse_node() {
  let runtime = Runtime::parse("node").unwrap();
  assert_eq!(runtime.name(), "node");
  assert_eq!(runtime.version(), "latest");

  let runtime = Runtime::parse("node@18").unwrap();
  assert_eq!(runtime.name(), "node");
  assert_eq!(runtime.version(), "18");

  let runtime = Runtime::parse("node@20.5.0").unwrap();
  assert_eq!(runtime.name(), "node");
  assert_eq!(runtime.version(), "20.5.0");
}

#[test]
fn test_runtime_parse_invalid() {
  assert!(Runtime::parse("invalid@1.0").is_err());
  assert!(Runtime::parse("").is_err());
}

#[test]
fn test_runtime_parse_python() {
  let runtime = Runtime::parse("python").unwrap();
  assert_eq!(runtime.name(), "python");
  assert_eq!(runtime.version(), "latest");

  let runtime = Runtime::parse("python@3.12").unwrap();
  assert_eq!(runtime.name(), "python");
  assert_eq!(runtime.version(), "3.12");

  let runtime = Runtime::parse("py@3.11.5").unwrap();
  assert_eq!(runtime.name(), "python");
  assert_eq!(runtime.version(), "3.11.5");
}

#[test]
fn test_runtime_default() {
  let runtime = Runtime::default();
  assert_eq!(runtime.name(), "bun");
  assert_eq!(runtime.version(), "latest");
}

#[test]
fn test_runtime_manager_new() {
  let result = RuntimeManager::new();
  assert!(result.is_ok());
}

#[test]
fn test_runtime_manager_paths() {
  let runtime_manager = RuntimeManager::new().unwrap();

  let bun_runtime = Runtime::Bun("1.0.1".to_string());
  let node_runtime = Runtime::Node("18.0.0".to_string());

  let bun_versions_dir = runtime_manager.get_runtime_versions_dir(&bun_runtime);
  let node_versions_dir = runtime_manager.get_runtime_versions_dir(&node_runtime);

  assert!(bun_versions_dir.to_string_lossy().contains("bun"));
  assert!(node_versions_dir.to_string_lossy().contains("node"));

  let bun_path = runtime_manager.get_runtime_path(&bun_runtime);
  let node_path = runtime_manager.get_runtime_path(&node_runtime);

  assert!(bun_path.to_string_lossy().ends_with("bun"));
  assert!(node_path.to_string_lossy().ends_with("node"));
}

#[test]
fn test_runtime_manager_npm_path() {
  let runtime_manager = RuntimeManager::new().unwrap();

  let bun_runtime = Runtime::Bun("1.0.1".to_string());
  let node_runtime = Runtime::Node("18.0.0".to_string());

  // Bun doesn't have npm
  assert_eq!(runtime_manager.get_npm_path(&bun_runtime), None);

  // Node should have npm path (even if not installed yet)
  let npm_path = runtime_manager.get_npm_path(&node_runtime);
  if let Some(path) = npm_path {
    assert!(path.to_string_lossy().contains("npm"));
  }
}

#[test]
fn test_runtime_manager_version_installed() {
  let runtime_manager = RuntimeManager::new().unwrap();

  let runtime = Runtime::Bun("999.999.999".to_string()); // Unlikely to exist
  assert!(!runtime_manager.is_version_installed(&runtime));
}

// Note: We don't test actual installation in unit tests as it requires network access
// and can be slow. These would be better as integration tests.

#[cfg(test)]
mod runtime_clone_tests {
  use super::*;

  #[test]
  fn test_runtime_clone() {
    let runtime = Runtime::Bun("1.0.1".to_string());
    let cloned = runtime.clone();

    assert_eq!(runtime.name(), cloned.name());
    assert_eq!(runtime.version(), cloned.version());
  }

  #[test]
  fn test_runtime_debug() {
    let runtime = Runtime::Node("18.0.0".to_string());
    let debug_str = format!("{:?}", runtime);

    assert!(debug_str.contains("Node"));
    assert!(debug_str.contains("18.0.0"));
  }
}

// Integration tests for version listing (requires network access)
#[cfg(test)]
mod version_listing_tests {
  use super::*;

  #[tokio::test]
  #[ignore] // Run with: cargo test --test runtime_tests -- --ignored
  async fn test_list_python_versions() {
    let manager = RuntimeManager::new().unwrap();
    let runtime = Runtime::Python("latest".to_string());

    let result = manager.list_available_versions(&runtime).await;
    assert!(
      result.is_ok(),
      "Failed to fetch Python versions: {:?}",
      result.err()
    );

    let versions = result.unwrap();
    assert!(
      !versions.is_empty(),
      "Should have at least one Python version"
    );

    // Verify version format (should be like "3.12.6")
    for version in &versions {
      let parts: Vec<&str> = version.split('.').collect();
      assert!(
        parts.len() >= 2,
        "Version should have at least major.minor: {}",
        version
      );

      // First part should be numeric
      assert!(
        parts[0].parse::<u32>().is_ok(),
        "Major version should be numeric: {}",
        version
      );
      assert!(
        parts[1].parse::<u32>().is_ok(),
        "Minor version should be numeric: {}",
        version
      );
    }

    // Should include common Python versions
    let has_3_12 = versions.iter().any(|v| v.starts_with("3.12"));
    let has_3_11 = versions.iter().any(|v| v.starts_with("3.11"));
    assert!(
      has_3_12 || has_3_11,
      "Should include Python 3.11 or 3.12 versions"
    );

    println!("Found {} Python versions", versions.len());
    println!("Sample versions: {:?}", &versions[..versions.len().min(5)]);
  }

  #[tokio::test]
  #[ignore]
  async fn test_list_bun_versions() {
    let manager = RuntimeManager::new().unwrap();
    let runtime = Runtime::Bun("latest".to_string());

    let result = manager.list_available_versions(&runtime).await;
    assert!(
      result.is_ok(),
      "Failed to fetch Bun versions: {:?}",
      result.err()
    );

    let versions = result.unwrap();
    assert!(!versions.is_empty(), "Should have at least one Bun version");

    // Verify version format (should be like "1.0.1")
    for version in &versions {
      let parts: Vec<&str> = version.split('.').collect();
      assert!(
        parts.len() >= 2,
        "Version should have at least major.minor: {}",
        version
      );

      assert!(
        parts[0].parse::<u32>().is_ok(),
        "Major version should be numeric: {}",
        version
      );
      assert!(
        parts[1].parse::<u32>().is_ok(),
        "Minor version should be numeric: {}",
        version
      );
    }

    // All Bun versions should start with "1."
    let all_v1 = versions.iter().all(|v| v.starts_with("1."));
    assert!(all_v1, "All Bun versions should be 1.x");

    println!("Found {} Bun versions", versions.len());
    println!("Sample versions: {:?}", &versions[..versions.len().min(5)]);
  }

  #[tokio::test]
  #[ignore]
  async fn test_list_node_versions() {
    let manager = RuntimeManager::new().unwrap();
    let runtime = Runtime::Node("latest".to_string());

    let result = manager.list_available_versions(&runtime).await;
    assert!(
      result.is_ok(),
      "Failed to fetch Node versions: {:?}",
      result.err()
    );

    let versions = result.unwrap();
    assert!(
      !versions.is_empty(),
      "Should have at least one Node version"
    );

    // Verify version format (should be like "20.5.0")
    for version in &versions {
      let parts: Vec<&str> = version.split('.').collect();
      assert!(
        parts.len() >= 2,
        "Version should have at least major.minor: {}",
        version
      );

      assert!(
        parts[0].parse::<u32>().is_ok(),
        "Major version should be numeric: {}",
        version
      );
      assert!(
        parts[1].parse::<u32>().is_ok(),
        "Minor version should be numeric: {}",
        version
      );
    }

    // Should include current or recent versions (20+)
    let has_recent = versions.iter().any(|v| {
      let major: u32 = v
        .split('.')
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
      major >= 20
    });
    assert!(has_recent, "Should include recent Node versions (20+)");

    println!("Found {} Node versions", versions.len());
    println!("Sample versions: {:?}", &versions[..versions.len().min(5)]);
  }

  #[tokio::test]
  #[ignore]
  async fn test_get_latest_python_version() {
    let manager = RuntimeManager::new().unwrap();
    let runtime = Runtime::Python("latest".to_string());

    let resolved = manager.resolve_latest_to_actual(&runtime).await;
    assert!(
      resolved.is_ok(),
      "Failed to resolve latest Python version: {:?}",
      resolved.err()
    );

    let resolved_runtime = resolved.unwrap();
    assert_eq!(resolved_runtime.name(), "python");
    assert_ne!(
      resolved_runtime.version(),
      "latest",
      "Should resolve to actual version"
    );

    // Should be a valid version number
    let version = resolved_runtime.version();
    let parts: Vec<&str> = version.split('.').collect();
    assert!(parts.len() >= 2, "Version should have at least major.minor");

    println!("Latest Python version: {}", version);
  }

  #[tokio::test]
  #[ignore]
  async fn test_get_latest_bun_version() {
    let manager = RuntimeManager::new().unwrap();
    let runtime = Runtime::Bun("latest".to_string());

    let resolved = manager.resolve_latest_to_actual(&runtime).await;
    assert!(
      resolved.is_ok(),
      "Failed to resolve latest Bun version: {:?}",
      resolved.err()
    );

    let resolved_runtime = resolved.unwrap();
    assert_eq!(resolved_runtime.name(), "bun");
    assert_ne!(
      resolved_runtime.version(),
      "latest",
      "Should resolve to actual version"
    );

    let version = resolved_runtime.version();
    let parts: Vec<&str> = version.split('.').collect();
    assert!(parts.len() >= 2, "Version should have at least major.minor");

    println!("Latest Bun version: {}", version);
  }

  #[tokio::test]
  #[ignore]
  async fn test_get_latest_node_version() {
    let manager = RuntimeManager::new().unwrap();
    let runtime = Runtime::Node("latest".to_string());

    let resolved = manager.resolve_latest_to_actual(&runtime).await;
    assert!(
      resolved.is_ok(),
      "Failed to resolve latest Node version: {:?}",
      resolved.err()
    );

    let resolved_runtime = resolved.unwrap();
    assert_eq!(resolved_runtime.name(), "node");
    assert_ne!(
      resolved_runtime.version(),
      "latest",
      "Should resolve to actual version"
    );

    let version = resolved_runtime.version();
    let parts: Vec<&str> = version.split('.').collect();
    assert!(parts.len() >= 2, "Version should have at least major.minor");

    println!("Latest Node version: {}", version);
  }

  #[tokio::test]
  async fn test_resolve_non_latest_version() {
    let manager = RuntimeManager::new().unwrap();
    let runtime = Runtime::Node("18.0.0".to_string());

    let resolved = manager.resolve_latest_to_actual(&runtime).await;
    assert!(resolved.is_ok());

    let resolved_runtime = resolved.unwrap();
    assert_eq!(
      resolved_runtime.version(),
      "18.0.0",
      "Non-latest version should not change"
    );
  }
}

// URL validation tests
#[cfg(test)]
mod url_validation_tests {
  use realm::runtime::manager::validate_download_url;

  #[test]
  fn test_validate_download_url_https() {
    let allowed_hosts = vec!["github.com".to_string(), "nodejs.org".to_string()];

    let result = validate_download_url(
      "https://github.com/oven-sh/bun/releases/download/bun-v1.0.1/bun-darwin-x64.zip",
      &allowed_hosts,
    );
    assert!(result.is_ok());
  }

  #[test]
  fn test_validate_download_url_http_rejected() {
    let allowed_hosts = vec!["github.com".to_string()];

    let result = validate_download_url("http://github.com/malicious/file.zip", &allowed_hosts);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("HTTPS"));
  }

  #[test]
  fn test_validate_download_url_unauthorized_host() {
    let allowed_hosts = vec!["github.com".to_string()];

    let result = validate_download_url("https://evil.com/malware.zip", &allowed_hosts);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not allowed"));
  }

  #[test]
  fn test_validate_download_url_subdomain_allowed() {
    let allowed_hosts = vec!["github.com".to_string()];

    let result = validate_download_url(
      "https://api.github.com/repos/oven-sh/bun/releases/latest",
      &allowed_hosts,
    );
    assert!(result.is_ok());
  }

  #[test]
  fn test_validate_download_url_invalid_url() {
    let allowed_hosts = vec!["github.com".to_string()];

    let result = validate_download_url("not a url", &allowed_hosts);
    assert!(result.is_err());
  }
}

// Platform info tests
#[cfg(test)]
mod platform_tests {
  use realm::runtime::manager::get_platform_info;

  #[test]
  fn test_get_platform_info() {
    let result = get_platform_info();
    assert!(
      result.is_ok(),
      "Should get platform info on supported platforms"
    );

    let (os, arch) = result.unwrap();

    // OS should be darwin or linux
    assert!(
      os == "darwin" || os == "linux",
      "OS should be darwin or linux, got: {}",
      os
    );

    // Arch should be x64 or arm64
    assert!(
      arch == "x64" || arch == "arm64",
      "Arch should be x64 or arm64, got: {}",
      arch
    );

    println!("Platform: {}-{}", os, arch);
  }
}
