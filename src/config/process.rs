use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProcessConfig {
    pub command: String,
    pub port: Option<u16>,
    #[serde(default)]
    pub routes: Vec<String>,
    pub working_directory: Option<String>,
}
