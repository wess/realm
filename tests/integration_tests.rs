use realm::config::{ProcessConfig, RealmConfig};
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_full_config_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("realm.yml");
    
    // Create a complete realm configuration
    let mut env = HashMap::new();
    env.insert("NODE_ENV".to_string(), "development".to_string());
    env.insert("PORT".to_string(), "8000".to_string());
    
    let mut processes = HashMap::new();
    processes.insert(
        "frontend".to_string(),
        ProcessConfig {
            command: "bun run dev".to_string(),
            port: Some(4000),
            routes: vec!["/".to_string(), "/assets/*".to_string(), "/static/*".to_string()],
            working_directory: Some("frontend".to_string()),
        },
    );
    processes.insert(
        "backend".to_string(),
        ProcessConfig {
            command: "bun run server".to_string(),
            port: Some(4001),
            routes: vec!["/api/*".to_string(), "/health".to_string()],
            working_directory: Some("backend".to_string()),
        },
    );
    processes.insert(
        "docs".to_string(),
        ProcessConfig {
            command: "bun run docs".to_string(),
            port: Some(4002),
            routes: vec!["/docs/*".to_string()],
            working_directory: Some("docs".to_string()),
        },
    );
    
    let config = RealmConfig {
        env,
        env_file: Some(".env".to_string()),
        processes,
        proxy_port: 8000,
    };
    
    // Save the configuration
    config.save(&config_path).unwrap();
    
    // Verify the file was created
    assert!(config_path.exists());
    
    // Load the configuration back
    let loaded_config = RealmConfig::load(&config_path).unwrap();
    
    // Verify all fields match
    assert_eq!(config.proxy_port, loaded_config.proxy_port);
    assert_eq!(config.env_file, loaded_config.env_file);
    assert_eq!(config.env.len(), loaded_config.env.len());
    assert_eq!(config.processes.len(), loaded_config.processes.len());
    
    // Verify environment variables
    assert_eq!(loaded_config.env.get("NODE_ENV"), Some(&"development".to_string()));
    assert_eq!(loaded_config.env.get("PORT"), Some(&"8000".to_string()));
    
    // Verify process configurations
    let frontend = loaded_config.processes.get("frontend").unwrap();
    assert_eq!(frontend.command, "bun run dev");
    assert_eq!(frontend.port, Some(4000));
    assert_eq!(frontend.routes.len(), 3);
    assert_eq!(frontend.working_directory, Some("frontend".to_string()));
    
    let backend = loaded_config.processes.get("backend").unwrap();
    assert_eq!(backend.command, "bun run server");
    assert_eq!(backend.port, Some(4001));
    assert_eq!(backend.routes.len(), 2);
    
    // Verify the YAML structure
    let yaml_content = fs::read_to_string(&config_path).unwrap();
    assert!(yaml_content.contains("proxy_port: 8000"));
    assert!(yaml_content.contains("NODE_ENV: development"));
    assert!(yaml_content.contains("frontend:"));
    assert!(yaml_content.contains("backend:"));
    assert!(yaml_content.contains("docs:"));
}

#[test]
fn test_minimal_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("minimal.yml");
    
    // Create minimal configuration
    let config = RealmConfig {
        env: HashMap::new(),
        env_file: None,
        processes: HashMap::new(),
        proxy_port: 3000,
    };
    
    // Save and load
    config.save(&config_path).unwrap();
    let loaded_config = RealmConfig::load(&config_path).unwrap();
    
    assert_eq!(loaded_config.proxy_port, 3000);
    assert!(loaded_config.env.is_empty());
    assert!(loaded_config.processes.is_empty());
    assert_eq!(loaded_config.env_file, None);
}

#[test]
fn test_config_with_complex_routes() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("complex.yml");
    
    let mut processes = HashMap::new();
    processes.insert(
        "spa".to_string(),
        ProcessConfig {
            command: "serve -s build".to_string(),
            port: Some(5000),
            routes: vec![
                "/".to_string(),
                "/app/*".to_string(),
                "/dashboard/*".to_string(),
                "/assets/*".to_string(),
                "/static/*".to_string(),
            ],
            working_directory: None,
        },
    );
    processes.insert(
        "api_v1".to_string(),
        ProcessConfig {
            command: "node api-v1.js".to_string(),
            port: Some(5001),
            routes: vec![
                "/api/v1/*".to_string(),
                "/v1/*".to_string(),
            ],
            working_directory: Some("api/v1".to_string()),
        },
    );
    processes.insert(
        "api_v2".to_string(),
        ProcessConfig {
            command: "node api-v2.js".to_string(),
            port: Some(5002),
            routes: vec![
                "/api/v2/*".to_string(),
                "/v2/*".to_string(),
            ],
            working_directory: Some("api/v2".to_string()),
        },
    );
    
    let config = RealmConfig {
        env: HashMap::new(),
        env_file: Some(".env.local".to_string()),
        processes,
        proxy_port: 9000,
    };
    
    config.save(&config_path).unwrap();
    let loaded_config = RealmConfig::load(&config_path).unwrap();
    
    assert_eq!(loaded_config.processes.len(), 3);
    assert_eq!(loaded_config.proxy_port, 9000);
    assert_eq!(loaded_config.env_file, Some(".env.local".to_string()));
    
    // Verify route complexity
    let spa_routes = &loaded_config.processes.get("spa").unwrap().routes;
    assert_eq!(spa_routes.len(), 5);
    assert!(spa_routes.contains(&"/dashboard/*".to_string()));
    
    let api_v1_routes = &loaded_config.processes.get("api_v1").unwrap().routes;
    assert_eq!(api_v1_routes.len(), 2);
    assert!(api_v1_routes.contains(&"/api/v1/*".to_string()));
}

#[test]
fn test_env_file_integration() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("realm.yml");
    let env_path = temp_dir.path().join(".env.test");
    
    // Create env file
    let env_content = r#"
DATABASE_URL=postgresql://localhost/test
REDIS_URL=redis://localhost:6379
API_SECRET=super-secret-key
ENVIRONMENT=test
"#;
    fs::write(&env_path, env_content).unwrap();
    
    // Create config that references the env file
    let config = RealmConfig {
        env: HashMap::from([
            ("INLINE_VAR".to_string(), "inline_value".to_string()),
        ]),
        env_file: Some(".env.test".to_string()),
        processes: HashMap::new(),
        proxy_port: 8000,
    };
    
    config.save(&config_path).unwrap();
    
    // Verify the saved config references the env file
    let yaml_content = fs::read_to_string(&config_path).unwrap();
    assert!(yaml_content.contains("env_file: .env.test"));
    assert!(yaml_content.contains("INLINE_VAR: inline_value"));
    
    let loaded_config = RealmConfig::load(&config_path).unwrap();
    assert_eq!(loaded_config.env_file, Some(".env.test".to_string()));
    assert_eq!(loaded_config.env.get("INLINE_VAR"), Some(&"inline_value".to_string()));
}