use crate::errors::{RealmError, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::activation::RealmEnvironment;
use crate::bundle::Bundler;
use crate::cache::CacheManager;
use crate::config::RealmConfig;
use crate::env::EnvManager;
use crate::process::ProcessManager;
use crate::proxy::ProxyServer;
use crate::runtime::{Runtime, RuntimeManager};
use crate::templates::TemplateManager;

#[derive(Parser)]
#[command(name = "realm")]
#[command(about = "Full-stack development environment CLI with built-in proxy")]
#[command(version = env!("REALM_VERSION"))]
pub struct Cli {
  #[command(subcommand)]
  pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
  /// Initialize a new realm environment
  Init {
    /// Path for the realm environment (default: .venv)
    #[arg(default_value = ".venv")]
    path: PathBuf,

    /// Runtime to use (bun, node, bun@1.0.1, node@18)
    #[arg(long, default_value = "bun")]
    runtime: String,

    /// Template to use for project scaffolding
    #[arg(long)]
    template: Option<String>,
  },

  /// Start all processes and proxy server
  Start,

  /// Stop all processes and proxy server
  Stop,

  /// Start proxy server only
  Proxy,

  /// Create deployment bundle
  Bundle,

  /// Create a new template from current project
  Create {
    /// Name of the template to create
    #[arg(long)]
    template: String,
  },

  /// Template management commands
  Templates {
    #[command(subcommand)]
    command: TemplateCommands,
  },

  /// List available runtime versions
  List {
    /// Runtime to list versions for (bun, node, python)
    #[arg(long)]
    runtime: String,
  },

  /// Cache management commands
  Cache {
    #[command(subcommand)]
    command: CacheCommands,
  },
}

#[derive(Subcommand)]
pub enum TemplateCommands {
  /// List available templates
  List,
}

#[derive(Subcommand)]
pub enum CacheCommands {
  /// Clear all cached runtime versions
  Clear,
}

pub struct CliHandler {
  template_manager: TemplateManager,
  runtime_manager: RuntimeManager,
}

impl CliHandler {
  pub fn new() -> Result<Self> {
    Ok(Self {
      template_manager: TemplateManager::new()?,
      runtime_manager: RuntimeManager::new()?,
    })
  }

  pub async fn handle_command(&self, command: Commands) -> Result<()> {
    match command {
      Commands::Init {
        path,
        runtime,
        template,
      } => self.handle_init(path, runtime, template).await,
      Commands::Start => self.handle_start().await,
      Commands::Stop => self.handle_stop().await,
      Commands::Proxy => self.handle_proxy().await,
      Commands::Bundle => self.handle_bundle().await,
      Commands::Create { template } => self.handle_create_template(template).await,
      Commands::Templates { command } => self.handle_templates(command).await,
      Commands::List { runtime } => self.handle_list(runtime).await,
      Commands::Cache { command } => self.handle_cache(command).await,
    }
  }

  async fn handle_init(
    &self,
    path: PathBuf,
    runtime_spec: String,
    template: Option<String>,
  ) -> Result<()> {
    println!("🏗️  Initializing realm environment...");

    // Parse runtime specification
    let mut runtime = Runtime::parse(&runtime_spec)?;

    // Check if we can use system-installed runtime
    // Only use system runtime if user requested "latest" or didn't specify version
    let use_system = runtime.version() == "latest" && self.runtime_manager.is_available_on_system(&runtime);

    if use_system {
      println!("✅ Using system-installed {} (found in PATH)", runtime.name());
    } else {
      // Resolve "latest" to actual version if needed for .realm installation
      if runtime.version() == "latest" {
        runtime = self.runtime_manager.resolve_latest_to_actual(&runtime).await?;
      }

      // Install runtime if not already installed in ~/.realm
      if !self.runtime_manager.is_version_installed(&runtime) {
        println!("📦 Getting {} {}...", runtime.name(), runtime.version());
        self.runtime_manager.install_version(&runtime).await?;
      } else {
        println!("✅ {} {} already installed", runtime.name(), runtime.version());
      }
    }

    // Create project from template if specified
    if let Some(template_name) = &template {
      let project_dir = std::env::current_dir()?.join("project");
      println!("🎯 Creating project from template '{template_name}'...");
      self
        .template_manager
        .init_from_template(template_name, &project_dir)?;
      std::env::set_current_dir(&project_dir)?;
    }

    // Initialize realm environment
    let realm_env = RealmEnvironment::init(&path)?;

    // Set up Python-specific isolation if using Python runtime
    realm_env.setup_python_isolation(&runtime, &self.runtime_manager)?;

    println!("✅ Realm environment initialized!");
    println!("🎯 Runtime: {} {}", runtime.name(), runtime.version());
    if let Some(template_name) = template {
      println!("📄 Template: {template_name}");
    }
    println!();
    println!("Next steps:");
    println!("  source {}/bin/activate", path.display());
    println!("  realm start");

    Ok(())
  }

  async fn handle_start(&self) -> Result<()> {
    // Check if we're in an activated realm environment
    if std::env::var("REALM_ENV").is_err() {
      return Err(RealmError::ValidationError(
        "Not in an activated realm environment. Run 'source .venv/bin/activate' first.".to_string()
      ));
    }

    println!("🚀 Starting realm environment...");

    // Load configuration
    let config = RealmConfig::load("realm.yml")?;

    // Set up environment variables
    let mut env_manager = EnvManager::new();
    env_manager.load_from_map(&config.env);
    if let Some(env_file) = &config.env_file {
      env_manager.load_from_file(env_file)?;
    }
    env_manager.apply();

    // Create process manager
    let process_manager = ProcessManager::new();
    process_manager.load_processes(&config)?;

    // Start all processes
    println!("🔧 Starting processes...");
    process_manager.start_all()?;

    // Start proxy server
    println!("🌐 Starting proxy server...");
    let proxy_server = ProxyServer::new(config, process_manager);

    // This will run indefinitely
    proxy_server.start().await?;

    Ok(())
  }

  async fn handle_stop(&self) -> Result<()> {
    println!("🛑 Stopping realm environment...");

    // Load configuration
    let config = RealmConfig::load("realm.yml")?;

    // Create process manager and stop all processes
    let process_manager = ProcessManager::new();
    process_manager.load_processes(&config)?;
    process_manager.stop_all()?;

    println!("✅ All processes stopped");
    Ok(())
  }

  async fn handle_proxy(&self) -> Result<()> {
    println!("🌐 Starting proxy server...");

    // Load configuration
    let config = RealmConfig::load("realm.yml")?;

    // Create process manager (for route mapping)
    let process_manager = ProcessManager::new();
    process_manager.load_processes(&config)?;

    // Start proxy server
    let proxy_server = ProxyServer::new(config, process_manager);
    proxy_server.start().await?;

    Ok(())
  }

  async fn handle_bundle(&self) -> Result<()> {
    println!("📦 Creating deployment bundle...");

    // Load configuration
    let config = RealmConfig::load("realm.yml")?;

    // Create bundler and generate deployment artifacts
    let bundler = Bundler::new(config)?;
    bundler.bundle()?;

    Ok(())
  }

  async fn handle_create_template(&self, template_name: String) -> Result<()> {
    println!("🎨 Creating template '{template_name}'...");

    self
      .template_manager
      .create_template_from_current_dir(&template_name)?;

    Ok(())
  }

  async fn handle_templates(&self, command: TemplateCommands) -> Result<()> {
    match command {
      TemplateCommands::List => {
        println!("📄 Available templates:");

        // Create built-in templates if they don't exist
        let _ = self.template_manager.create_builtin_templates();

        let templates = self.template_manager.list_templates()?;
        if templates.is_empty() {
          println!("   No templates found");
        } else {
          for template in templates {
            println!("   • {template}");
          }
        }

        Ok(())
      }
    }
  }

  async fn handle_list(&self, runtime_spec: String) -> Result<()> {
    let runtime = Runtime::parse(&runtime_spec)?;

    println!("📦 Fetching available {} versions...", runtime.name());

    let versions = self.runtime_manager.list_available_versions(&runtime).await?;

    if versions.is_empty() {
      println!("   No versions found");
    } else {
      println!("\n   Available versions:");
      for version in versions {
        let installed_marker = if self.runtime_manager.is_version_installed(&Runtime::from_name_version(runtime.name(), &version)) {
          " (installed)"
        } else {
          ""
        };
        println!("   • {}{}", version, installed_marker);
      }
    }

    Ok(())
  }

  async fn handle_cache(&self, command: CacheCommands) -> Result<()> {
    match command {
      CacheCommands::Clear => {
        println!("🗑️  Clearing runtime version cache...");

        let cache_manager = CacheManager::new()?;
        cache_manager.clear_all()?;

        println!("✅ Cache cleared successfully");
        Ok(())
      }
    }
  }
}

impl Default for CliHandler {
  fn default() -> Self {
    Self::new().expect("Failed to create CliHandler")
  }
}
