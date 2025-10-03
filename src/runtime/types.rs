use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
pub enum Runtime {
  Bun(String),    // version
  Node(String),   // version
  Python(String), // version
}

impl Runtime {
  pub fn parse(runtime_spec: &str) -> Result<Self> {
    if runtime_spec.starts_with("bun") {
      let version = if runtime_spec.contains('@') {
        runtime_spec
          .split('@')
          .nth(1)
          .unwrap_or("latest")
          .to_string()
      } else {
        "latest".to_string()
      };
      Ok(Runtime::Bun(version))
    } else if runtime_spec.starts_with("node") {
      let version = if runtime_spec.contains('@') {
        runtime_spec
          .split('@')
          .nth(1)
          .unwrap_or("latest")
          .to_string()
      } else {
        "latest".to_string()
      };
      Ok(Runtime::Node(version))
    } else if runtime_spec.starts_with("python") || runtime_spec.starts_with("py") {
      let version = if runtime_spec.contains('@') {
        runtime_spec
          .split('@')
          .nth(1)
          .unwrap_or("latest")
          .to_string()
      } else {
        "latest".to_string()
      };
      Ok(Runtime::Python(version))
    } else {
      Err(anyhow!(
        "Unknown runtime: {}. Supported: bun, node, python",
        runtime_spec
      ))
    }
  }

  pub fn name(&self) -> &str {
    match self {
      Runtime::Bun(_) => "bun",
      Runtime::Node(_) => "node",
      Runtime::Python(_) => "python",
    }
  }

  pub fn version(&self) -> &str {
    match self {
      Runtime::Bun(v) | Runtime::Node(v) | Runtime::Python(v) => v,
    }
  }

  pub fn from_name_version(name: &str, version: &str) -> Self {
    match name {
      "bun" => Runtime::Bun(version.to_string()),
      "node" => Runtime::Node(version.to_string()),
      "python" => Runtime::Python(version.to_string()),
      _ => Runtime::Bun("latest".to_string()),
    }
  }
}

impl Default for Runtime {
  fn default() -> Self {
    Runtime::Bun("latest".to_string())
  }
}
