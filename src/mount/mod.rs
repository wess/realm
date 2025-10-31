use crate::errors::{RealmError, Result};
use colored::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct DetectedFeature {
  pub name: String,
  pub path: PathBuf,
  pub feature_type: FeatureType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FeatureType {
  RealmConfig,
  PackageJson,
  Requirements,
  EnvExample,
  DockerCompose,
  CargoToml,
  GoMod,
}

pub struct ProjectDetector {
  project_path: PathBuf,
}

impl ProjectDetector {
  pub fn new(project_path: PathBuf) -> Self {
    Self { project_path }
  }

  pub fn detect(&self) -> Result<Vec<DetectedFeature>> {
    let mut features = Vec::new();

    // Check for realm.yml in project root
    self.check_file("realm.yml", FeatureType::RealmConfig, &mut features)?;

    // Check for .env.example
    self.check_file(".env.example", FeatureType::EnvExample, &mut features)?;

    // Check for docker-compose.yml
    self.check_file("docker-compose.yml", FeatureType::DockerCompose, &mut features)?;

    // Recursively find package.json files
    self.find_files_recursive("package.json", FeatureType::PackageJson, &mut features)?;

    // Recursively find requirements.txt files
    self.find_files_recursive("requirements.txt", FeatureType::Requirements, &mut features)?;

    // Check for Cargo.toml
    self.check_file("Cargo.toml", FeatureType::CargoToml, &mut features)?;

    // Check for go.mod
    self.check_file("go.mod", FeatureType::GoMod, &mut features)?;

    Ok(features)
  }

  fn check_file(
    &self,
    filename: &str,
    feature_type: FeatureType,
    features: &mut Vec<DetectedFeature>,
  ) -> Result<()> {
    let file_path = self.project_path.join(filename);
    if file_path.exists() {
      features.push(DetectedFeature {
        name: filename.to_string(),
        path: file_path,
        feature_type,
      });
    }
    Ok(())
  }

  fn find_files_recursive(
    &self,
    filename: &str,
    feature_type: FeatureType,
    features: &mut Vec<DetectedFeature>,
  ) -> Result<()> {
    self.find_files_in_dir(&self.project_path, filename, feature_type, features, 0)?;
    Ok(())
  }

  fn find_files_in_dir(
    &self,
    dir: &Path,
    filename: &str,
    feature_type: FeatureType,
    features: &mut Vec<DetectedFeature>,
    depth: usize,
  ) -> Result<()> {
    // Limit recursion depth to avoid scanning too deep
    if depth > 3 {
      return Ok(());
    }

    // Skip common directories we don't want to scan
    let dir_name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if matches!(
      dir_name,
      "node_modules" | ".git" | "target" | "dist" | "build" | ".venv" | "venv"
    ) {
      return Ok(());
    }

    if let Ok(entries) = fs::read_dir(dir) {
      for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && path.file_name() == Some(std::ffi::OsStr::new(filename)) {
          features.push(DetectedFeature {
            name: filename.to_string(),
            path: path.clone(),
            feature_type: feature_type.clone(),
          });
        } else if path.is_dir() {
          self.find_files_in_dir(&path, filename, feature_type.clone(), features, depth + 1)?;
        }
      }
    }

    Ok(())
  }

  pub fn detect_runtime_from_realm_yml(&self) -> Result<Option<String>> {
    let realm_yml = self.project_path.join("realm.yml");
    if !realm_yml.exists() {
      return Ok(None);
    }

    let content = fs::read_to_string(&realm_yml)?;
    let config: serde_yaml::Value = serde_yaml::from_str(&content)
      .map_err(|e| RealmError::ValidationError(format!("Failed to parse realm.yml: {}", e)))?;

    // Try to extract runtime from config
    if let Some(runtime) = config.get("runtime").and_then(|r| r.as_str()) {
      return Ok(Some(runtime.to_string()));
    }

    Ok(None)
  }

  pub fn infer_runtime(&self, features: &[DetectedFeature]) -> String {
    // Check realm.yml first
    if let Ok(Some(runtime)) = self.detect_runtime_from_realm_yml() {
      return runtime;
    }

    // Infer from detected features
    for feature in features {
      match feature.feature_type {
        FeatureType::Requirements => return "python".to_string(),
        FeatureType::CargoToml => return "rust".to_string(),
        FeatureType::GoMod => return "go".to_string(),
        _ => {}
      }
    }

    // Default to bun if package.json found
    if features
      .iter()
      .any(|f| f.feature_type == FeatureType::PackageJson)
    {
      return "bun".to_string();
    }

    // Ultimate fallback
    "bun".to_string()
  }
}

pub struct DependencyInstaller;

impl DependencyInstaller {
  pub fn install_node_dependencies(path: &Path) -> Result<()> {
    let package_dir = path.parent().unwrap_or(Path::new("."));

    println!(
      "   {} Installing dependencies in {}...",
      "→".cyan(),
      package_dir.display()
    );

    // Try bun first, fallback to npm
    let install_cmd = if Command::new("bun").arg("--version").output().is_ok() {
      Command::new("bun")
        .arg("install")
        .current_dir(package_dir)
        .output()
    } else {
      Command::new("npm")
        .arg("install")
        .current_dir(package_dir)
        .output()
    };

    match install_cmd {
      Ok(output) => {
        if output.status.success() {
          println!("   {} Installed dependencies", "✓".green());
          Ok(())
        } else {
          Err(RealmError::ValidationError(format!(
            "Failed to install dependencies: {}",
            String::from_utf8_lossy(&output.stderr)
          )))
        }
      }
      Err(e) => Err(RealmError::ValidationError(format!(
        "Failed to run package manager: {}",
        e
      ))),
    }
  }

  pub fn install_python_dependencies(path: &Path) -> Result<()> {
    let requirements_dir = path.parent().unwrap_or(Path::new("."));

    println!(
      "   {} Installing Python dependencies from {}...",
      "→".cyan(),
      path.display()
    );

    let output = Command::new("pip")
      .args(["install", "-r", path.to_str().unwrap()])
      .current_dir(requirements_dir)
      .output()
      .map_err(|e| {
        RealmError::ValidationError(format!("Failed to run pip install: {}", e))
      })?;

    if output.status.success() {
      println!("   {} Installed Python dependencies", "✓".green());
      Ok(())
    } else {
      Err(RealmError::ValidationError(format!(
        "Failed to install Python dependencies: {}",
        String::from_utf8_lossy(&output.stderr)
      )))
    }
  }
}

pub fn copy_env_example(project_path: &Path) -> Result<()> {
  let env_example = project_path.join(".env.example");
  let env_file = project_path.join(".env");

  if env_file.exists() {
    println!("   {} .env already exists, skipping", "→".yellow());
    return Ok(());
  }

  println!("   {} Copying .env.example → .env", "→".cyan());
  fs::copy(&env_example, &env_file)?;
  println!("   {} Created .env file", "✓".green());

  Ok(())
}
