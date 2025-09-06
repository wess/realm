use realm::runtime::{Runtime, RuntimeManager};

#[test]
fn test_runtime_parse_bun() {
    let runtime = Runtime::parse("bun").unwrap();
    assert_eq!(runtime.name(), "bun");
    assert_eq!(runtime.version(), "latest");
    
    let runtime = Runtime::parse("bun@1.0.0").unwrap();
    assert_eq!(runtime.name(), "bun");
    assert_eq!(runtime.version(), "1.0.0");
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
    assert!(Runtime::parse("python").is_err());
    assert!(Runtime::parse("invalid@1.0").is_err());
    assert!(Runtime::parse("").is_err());
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
    
    let bun_runtime = Runtime::Bun("1.0.0".to_string());
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
    
    let bun_runtime = Runtime::Bun("1.0.0".to_string());
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
        let runtime = Runtime::Bun("1.0.0".to_string());
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