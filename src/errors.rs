use std::fmt;

#[derive(Debug, Clone)]
pub enum RealmError {
  ConfigError(ConfigError),
  ProcessError(ProcessError),
  ProxyError(ProxyError),
  RuntimeError(RuntimeError),
  TemplateError(TemplateError),
  IoError(String),
  NetworkError(String),
  ValidationError(String),
}

#[derive(Debug, Clone)]
pub enum ConfigError {
  FileNotFound(String),
  ParseError(String),
  InvalidFormat(String),
  MissingField(String),
}

#[derive(Debug, Clone)]
pub enum ProcessError {
  StartFailed(String),
  StopFailed(String),
  NotFound(String),
  AlreadyRunning(String),
  CommandParseError(String),
  PermissionDenied(String),
}

#[derive(Debug, Clone)]
pub enum ProxyError {
  BindFailed(String),
  RouteNotFound(String),
  UpstreamError(String),
  InvalidPort(u16),
  RequestForwardError(String),
}

#[derive(Debug, Clone)]
pub enum RuntimeError {
  NotInstalled(String),
  InstallationFailed(String),
  DownloadFailed(String),
  UnsupportedPlatform(String),
  InvalidVersion(String),
  ExtractionFailed(String),
}

#[derive(Debug, Clone)]
pub enum TemplateError {
  NotFound(String),
  CreationFailed(String),
  InvalidTemplate(String),
  FileSystemError(String),
}

impl fmt::Display for RealmError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      RealmError::ConfigError(e) => write!(f, "Configuration error: {e}"),
      RealmError::ProcessError(e) => write!(f, "Process error: {e}"),
      RealmError::ProxyError(e) => write!(f, "Proxy error: {e}"),
      RealmError::RuntimeError(e) => write!(f, "Runtime error: {e}"),
      RealmError::TemplateError(e) => write!(f, "Template error: {e}"),
      RealmError::IoError(msg) => write!(f, "IO error: {msg}"),
      RealmError::NetworkError(msg) => write!(f, "Network error: {msg}"),
      RealmError::ValidationError(msg) => write!(f, "Validation error: {msg}"),
    }
  }
}

impl fmt::Display for ConfigError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ConfigError::FileNotFound(path) => write!(f, "Config file not found: {path}"),
      ConfigError::ParseError(msg) => write!(f, "Failed to parse config: {msg}"),
      ConfigError::InvalidFormat(msg) => write!(f, "Invalid config format: {msg}"),
      ConfigError::MissingField(field) => write!(f, "Missing required field: {field}"),
    }
  }
}

impl fmt::Display for ProcessError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ProcessError::StartFailed(name) => write!(f, "Failed to start process: {name}"),
      ProcessError::StopFailed(name) => write!(f, "Failed to stop process: {name}"),
      ProcessError::NotFound(name) => write!(f, "Process not found: {name}"),
      ProcessError::AlreadyRunning(name) => write!(f, "Process already running: {name}"),
      ProcessError::CommandParseError(cmd) => write!(f, "Invalid command: {cmd}"),
      ProcessError::PermissionDenied(name) => write!(f, "Permission denied for process: {name}"),
    }
  }
}

impl fmt::Display for ProxyError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ProxyError::BindFailed(addr) => write!(f, "Failed to bind to address: {addr}"),
      ProxyError::RouteNotFound(path) => write!(f, "No route found for path: {path}"),
      ProxyError::UpstreamError(msg) => write!(f, "Upstream error: {msg}"),
      ProxyError::InvalidPort(port) => write!(f, "Invalid port: {port}"),
      ProxyError::RequestForwardError(msg) => write!(f, "Request forwarding failed: {msg}"),
    }
  }
}

impl fmt::Display for RuntimeError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      RuntimeError::NotInstalled(runtime) => write!(f, "Runtime not installed: {runtime}"),
      RuntimeError::InstallationFailed(runtime) => write!(f, "Installation failed: {runtime}"),
      RuntimeError::DownloadFailed(url) => write!(f, "Download failed: {url}"),
      RuntimeError::UnsupportedPlatform(platform) => write!(f, "Unsupported platform: {platform}"),
      RuntimeError::InvalidVersion(version) => write!(f, "Invalid version: {version}"),
      RuntimeError::ExtractionFailed(msg) => write!(f, "Extraction failed: {msg}"),
    }
  }
}

impl fmt::Display for TemplateError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      TemplateError::NotFound(name) => write!(f, "Template not found: {name}"),
      TemplateError::CreationFailed(name) => write!(f, "Template creation failed: {name}"),
      TemplateError::InvalidTemplate(name) => write!(f, "Invalid template: {name}"),
      TemplateError::FileSystemError(msg) => write!(f, "File system error: {msg}"),
    }
  }
}

impl std::error::Error for RealmError {}
impl std::error::Error for ConfigError {}
impl std::error::Error for ProcessError {}
impl std::error::Error for ProxyError {}
impl std::error::Error for RuntimeError {}
impl std::error::Error for TemplateError {}

pub type Result<T> = std::result::Result<T, RealmError>;

impl From<std::io::Error> for RealmError {
  fn from(err: std::io::Error) -> Self {
    RealmError::IoError(err.to_string())
  }
}

impl From<reqwest::Error> for RealmError {
  fn from(err: reqwest::Error) -> Self {
    RealmError::NetworkError(err.to_string())
  }
}

impl From<serde_yaml::Error> for RealmError {
  fn from(err: serde_yaml::Error) -> Self {
    RealmError::ConfigError(ConfigError::ParseError(err.to_string()))
  }
}

impl From<anyhow::Error> for RealmError {
  fn from(err: anyhow::Error) -> Self {
    RealmError::ValidationError(err.to_string())
  }
}