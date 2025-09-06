use crate::config::ProcessConfig;
use std::process::Child;

#[derive(Debug)]
pub struct ProcessInfo {
    pub name: String,
    pub config: ProcessConfig,
    pub child: Option<Child>,
    pub port: Option<u16>,
}
