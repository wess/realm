use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateManifest {
  pub name: String,
  #[serde(default)]
  pub description: String,
  #[serde(default)]
  pub variables: Vec<TemplateVariable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateVariable {
  pub name: String,
  pub prompt: String,
  #[serde(default)]
  pub default: String,
}

impl TemplateManifest {
  pub fn from_yaml_str(yaml: &str) -> Result<Self> {
    Ok(serde_yaml::from_str(yaml)?)
  }

  pub fn from_file(path: &std::path::Path) -> Result<Self> {
    let content = std::fs::read_to_string(path)?;
    Self::from_yaml_str(&content)
  }
}
