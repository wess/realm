use crate::config::RealmConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Template {
    pub name: String,
    pub description: String,
    pub version: String,
    pub files: Vec<TemplateFile>,
    pub realm_config: RealmConfig,
    #[serde(default)]
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TemplateFile {
    pub path: String,
    pub content: String,
    #[serde(default)]
    pub executable: bool,
}
