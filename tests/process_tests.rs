use realm::config::{ProcessConfig, RealmConfig};
use realm::process::ProcessManager;
use std::collections::HashMap;

#[test]
fn test_process_manager_new() {
  let process_manager = ProcessManager::new();
  assert!(process_manager.list_processes().is_empty());
}

#[test]
fn test_process_manager_load_processes() {
  let process_manager = ProcessManager::new();

  let mut processes = HashMap::new();
  processes.insert(
    "frontend".to_string(),
    ProcessConfig {
      command: "bun run dev".to_string(),
      port: Some(4000),
      routes: vec!["/".to_string()],
      working_directory: Some("frontend".to_string()),
    },
  );
  processes.insert(
    "backend".to_string(),
    ProcessConfig {
      command: "bun run server".to_string(),
      port: Some(4001),
      routes: vec!["/api/*".to_string()],
      working_directory: Some("backend".to_string()),
    },
  );

  let config = RealmConfig {
    env: HashMap::new(),
    env_file: Some(".env".to_string()),
    processes,
    proxy_port: 8000,
  };

  let result = process_manager.load_processes(&config);
  assert!(result.is_ok());

  let process_list = process_manager.list_processes();
  assert_eq!(process_list.len(), 2);
  assert!(process_list.contains(&"frontend".to_string()));
  assert!(process_list.contains(&"backend".to_string()));
}

#[test]
fn test_process_manager_get_process_port() {
  let process_manager = ProcessManager::new();

  let mut processes = HashMap::new();
  processes.insert(
    "web".to_string(),
    ProcessConfig {
      command: "npm start".to_string(),
      port: Some(3000),
      routes: vec!["/".to_string()],
      working_directory: None,
    },
  );

  let config = RealmConfig {
    env: HashMap::new(),
    env_file: None,
    processes,
    proxy_port: 8000,
  };

  process_manager.load_processes(&config).unwrap();

  assert_eq!(process_manager.get_process_port("web"), Some(3000));
  assert_eq!(process_manager.get_process_port("nonexistent"), None);
}

#[test]
fn test_process_manager_get_process_routes() {
  let process_manager = ProcessManager::new();

  let mut processes = HashMap::new();
  processes.insert(
    "api".to_string(),
    ProcessConfig {
      command: "node server.js".to_string(),
      port: Some(4000),
      routes: vec!["/api/*".to_string(), "/health".to_string()],
      working_directory: None,
    },
  );

  let config = RealmConfig {
    env: HashMap::new(),
    env_file: None,
    processes,
    proxy_port: 8000,
  };

  process_manager.load_processes(&config).unwrap();

  let routes = process_manager.get_process_routes("api");
  assert_eq!(routes.len(), 2);
  assert!(routes.contains(&"/api/*".to_string()));
  assert!(routes.contains(&"/health".to_string()));

  let empty_routes = process_manager.get_process_routes("nonexistent");
  assert!(empty_routes.is_empty());
}

#[test]
fn test_process_manager_is_running() {
  let process_manager = ProcessManager::new();

  // Process that doesn't exist should not be running
  assert!(!process_manager.is_running("nonexistent"));

  // Load a process but don't start it
  let mut processes = HashMap::new();
  processes.insert(
    "test".to_string(),
    ProcessConfig {
      command: "echo test".to_string(),
      port: Some(3000),
      routes: vec!["/".to_string()],
      working_directory: None,
    },
  );

  let config = RealmConfig {
    env: HashMap::new(),
    env_file: None,
    processes,
    proxy_port: 8000,
  };

  process_manager.load_processes(&config).unwrap();

  // Process exists but is not running
  assert!(!process_manager.is_running("test"));
}

#[test]
fn test_process_config_defaults() {
  let config = ProcessConfig {
    command: "test command".to_string(),
    port: None,
    routes: vec![],
    working_directory: None,
  };

  assert_eq!(config.command, "test command");
  assert_eq!(config.port, None);
  assert!(config.routes.is_empty());
  assert_eq!(config.working_directory, None);
}

// Note: Testing actual process starting/stopping requires more complex setup
// and might be flaky in CI environments. These would be better as integration tests.

#[cfg(test)]
mod process_integration_tests {
  use super::*;
  use std::time::Duration;
  use tokio::time::sleep;

  // This test is ignored by default as it actually starts processes
  #[tokio::test]
  #[ignore]
  async fn test_start_and_stop_process() {
    let process_manager = ProcessManager::new();

    let mut processes = HashMap::new();
    processes.insert(
      "echo".to_string(),
      ProcessConfig {
        command: "sleep 5".to_string(), // Long running command
        port: None,
        routes: vec![],
        working_directory: None,
      },
    );

    let config = RealmConfig {
      env: HashMap::new(),
      env_file: None,
      processes,
      proxy_port: 8000,
    };

    process_manager.load_processes(&config).unwrap();

    // Start the process
    let result = process_manager.start_process("echo");
    assert!(result.is_ok());

    // Give it a moment to start
    sleep(Duration::from_millis(100)).await;

    // Should be running now
    assert!(process_manager.is_running("echo"));

    // Stop the process
    let result = process_manager.stop_process("echo");
    assert!(result.is_ok());

    // Give it a moment to stop
    sleep(Duration::from_millis(100)).await;

    // Should not be running now
    assert!(!process_manager.is_running("echo"));
  }
}
