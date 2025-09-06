use realm::config::{ProcessConfig, RealmConfig};
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_realm_config_default() {
  let config = RealmConfig::default();
  assert_eq!(config.proxy_port, 8000);
  assert!(config.env.is_empty());
  assert!(config.processes.is_empty());
  assert_eq!(config.env_file, Some(".env".to_string()));
}

#[test]
fn test_realm_config_serialization() {
  let mut processes = HashMap::new();
  processes.insert(
    "frontend".to_string(),
    ProcessConfig {
      command: "bun run dev".to_string(),
      port: Some(4000),
      routes: vec!["/".to_string(), "/assets/*".to_string()],
      working_directory: Some("frontend".to_string()),
    },
  );

  let mut env = HashMap::new();
  env.insert("NODE_ENV".to_string(), "development".to_string());

  let config = RealmConfig {
    env,
    env_file: Some(".env".to_string()),
    processes,
    proxy_port: 8000,
  };

  let yaml = serde_yaml::to_string(&config).unwrap();
  let deserialized: RealmConfig = serde_yaml::from_str(&yaml).unwrap();

  assert_eq!(config.proxy_port, deserialized.proxy_port);
  assert_eq!(config.env, deserialized.env);
  assert_eq!(config.processes.len(), deserialized.processes.len());
}

#[test]
fn test_realm_config_load_save() {
  let temp_dir = TempDir::new().unwrap();
  let config_path = temp_dir.path().join("realm.yml");

  let mut processes = HashMap::new();
  processes.insert(
    "backend".to_string(),
    ProcessConfig {
      command: "bun run server".to_string(),
      port: Some(4001),
      routes: vec!["/api/*".to_string()],
      working_directory: None,
    },
  );

  let config = RealmConfig {
    env: HashMap::new(),
    env_file: Some(".env".to_string()),
    processes,
    proxy_port: 3000,
  };

  // Save config
  config.save(&config_path).unwrap();
  assert!(config_path.exists());

  // Load config
  let loaded_config = RealmConfig::load(&config_path).unwrap();
  assert_eq!(config.proxy_port, loaded_config.proxy_port);
  assert_eq!(config.processes.len(), loaded_config.processes.len());
}

#[test]
fn test_process_config() {
  let process_config = ProcessConfig {
    command: "npm start".to_string(),
    port: Some(3000),
    routes: vec!["/".to_string()],
    working_directory: Some("./app".to_string()),
  };

  assert_eq!(process_config.command, "npm start");
  assert_eq!(process_config.port, Some(3000));
  assert_eq!(process_config.routes, vec!["/"]);
  assert_eq!(process_config.working_directory, Some("./app".to_string()));
}

#[test]
fn test_realm_config_load_or_default() {
  // Test with non-existent file
  let config = RealmConfig::load_or_default("non_existent.yml");
  assert_eq!(config.proxy_port, 8000);

  // Test with existing file
  let temp_dir = TempDir::new().unwrap();
  let config_path = temp_dir.path().join("test.yml");

  let yaml_content = r#"
proxy_port: 9000
env:
  TEST_VAR: "test_value"
processes:
  test:
    command: "echo test"
    port: 5000
    routes: ["/test"]
"#;

  fs::write(&config_path, yaml_content).unwrap();
  let config = RealmConfig::load_or_default(&config_path);
  assert_eq!(config.proxy_port, 9000);
  assert_eq!(config.env.get("TEST_VAR"), Some(&"test_value".to_string()));
}
