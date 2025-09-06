use realm::env::EnvManager;
use std::collections::HashMap;
use std::env;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_env_manager_new() {
  let env_manager = EnvManager::new();
  assert!(env_manager.vars().is_empty());
}

#[test]
fn test_env_manager_load_from_map() {
  let mut env_manager = EnvManager::new();

  let mut env_vars = HashMap::new();
  env_vars.insert("TEST_VAR1".to_string(), "value1".to_string());
  env_vars.insert("TEST_VAR2".to_string(), "value2".to_string());

  env_manager.load_from_map(&env_vars);

  assert_eq!(env_manager.get("TEST_VAR1"), Some(&"value1".to_string()));
  assert_eq!(env_manager.get("TEST_VAR2"), Some(&"value2".to_string()));
  assert_eq!(env_manager.vars().len(), 2);
}

#[test]
fn test_env_manager_load_from_file() {
  let temp_dir = TempDir::new().unwrap();
  let env_file = temp_dir.path().join(".env");

  let env_content = r#"
# This is a comment
NODE_ENV=development
API_KEY=secret123
DATABASE_URL="postgresql://user:pass@localhost/db"
EMPTY_VAR=

# Another comment
DEBUG=true
"#;

  fs::write(&env_file, env_content).unwrap();

  let mut env_manager = EnvManager::new();
  env_manager.load_from_file(&env_file).unwrap();

  assert_eq!(
    env_manager.get("NODE_ENV"),
    Some(&"development".to_string())
  );
  assert_eq!(env_manager.get("API_KEY"), Some(&"secret123".to_string()));
  assert_eq!(
    env_manager.get("DATABASE_URL"),
    Some(&"postgresql://user:pass@localhost/db".to_string())
  );
  assert_eq!(env_manager.get("EMPTY_VAR"), Some(&"".to_string()));
  assert_eq!(env_manager.get("DEBUG"), Some(&"true".to_string()));
  assert_eq!(env_manager.vars().len(), 5);
}

#[test]
fn test_env_manager_load_from_nonexistent_file() {
  let mut env_manager = EnvManager::new();
  let result = env_manager.load_from_file("nonexistent.env");

  // Should not error for non-existent file
  assert!(result.is_ok());
  assert!(env_manager.vars().is_empty());
}

#[test]
fn test_env_manager_set_and_get() {
  let mut env_manager = EnvManager::new();

  env_manager.set("CUSTOM_VAR".to_string(), "custom_value".to_string());
  assert_eq!(
    env_manager.get("CUSTOM_VAR"),
    Some(&"custom_value".to_string())
  );

  env_manager.set("CUSTOM_VAR".to_string(), "updated_value".to_string());
  assert_eq!(
    env_manager.get("CUSTOM_VAR"),
    Some(&"updated_value".to_string())
  );
}

#[test]
fn test_env_manager_apply() {
  let mut env_manager = EnvManager::new();

  // Set a unique test variable
  let test_var = "REALM_TEST_VAR_12345";
  env_manager.set(test_var.to_string(), "test_value".to_string());

  // Apply environment variables
  env_manager.apply();

  // Check if the variable was set in the actual environment
  assert_eq!(env::var(test_var).unwrap(), "test_value");

  // Clean up
  env::remove_var(test_var);
}

#[test]
fn test_env_file_parsing_edge_cases() {
  let temp_dir = TempDir::new().unwrap();
  let env_file = temp_dir.path().join(".env");

  let env_content = r#"
# Comment at start
VAR_WITH_EQUALS=value=with=equals
VAR_WITH_SPACES = value with spaces 
VAR_WITH_QUOTES='single quoted'
VAR_WITH_DOUBLE_QUOTES="double quoted"
VAR_EMPTY=
VAR_NO_VALUE

# Empty line above
FINAL_VAR=final
"#;

  fs::write(&env_file, env_content).unwrap();

  let mut env_manager = EnvManager::new();
  env_manager.load_from_file(&env_file).unwrap();

  assert_eq!(
    env_manager.get("VAR_WITH_EQUALS"),
    Some(&"value=with=equals".to_string())
  );
  assert_eq!(
    env_manager.get("VAR_WITH_SPACES"),
    Some(&"value with spaces".to_string())
  );
  assert_eq!(
    env_manager.get("VAR_WITH_QUOTES"),
    Some(&"single quoted".to_string())
  );
  assert_eq!(
    env_manager.get("VAR_WITH_DOUBLE_QUOTES"),
    Some(&"double quoted".to_string())
  );
  assert_eq!(env_manager.get("VAR_EMPTY"), Some(&"".to_string()));
  assert_eq!(env_manager.get("FINAL_VAR"), Some(&"final".to_string()));

  // VAR_NO_VALUE should not be loaded since it has no =
  assert_eq!(env_manager.get("VAR_NO_VALUE"), None);
}
