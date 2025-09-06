use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
pub enum Runtime {
    Bun(String),  // version
    Node(String), // version
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
        } else {
            Err(anyhow!(
                "Unknown runtime: {}. Supported: bun, node",
                runtime_spec
            ))
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Runtime::Bun(_) => "bun",
            Runtime::Node(_) => "node",
        }
    }

    pub fn version(&self) -> &str {
        match self {
            Runtime::Bun(v) | Runtime::Node(v) => v,
        }
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Runtime::Bun("latest".to_string())
    }
}
