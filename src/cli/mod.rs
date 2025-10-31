use crate::errors::{RealmError, Result};
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use colored::*;
use std::collections::HashMap;
use std::io;
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

fn parse_key_val(s: &str) -> Result<(String, String)> {
  let pos = s.find('=').ok_or_else(|| {
    RealmError::ValidationError(format!("invalid KEY=value: no `=` found in `{s}`"))
  })?;
  Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}

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

    /// Runtime to use (bun, node, bun@1.0.1, node@18, python@3.12)
    #[arg(long)]
    runtime: Option<String>,

    /// Template to use for project scaffolding
    #[arg(long)]
    template: Option<String>,

    /// Template variables (e.g., --var name=myapp --var author=john)
    #[arg(long = "var", value_parser = parse_key_val)]
    vars: Vec<(String, String)>,

    /// Skip interactive prompts and use defaults
    #[arg(long, short = 'y')]
    yes: bool,
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

  /// Generate shell completions
  Completions {
    /// Shell to generate completions for
    #[arg(value_enum)]
    shell: clap_complete::Shell,
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
  pub async fn new() -> Result<Self> {
    let mut runtime_manager = RuntimeManager::new()?;
    runtime_manager.init().await?;

    Ok(Self {
      template_manager: TemplateManager::new()?,
      runtime_manager,
    })
  }

  pub async fn handle_command(&self, command: Commands) -> Result<()> {
    match command {
      Commands::Init {
        path,
        runtime,
        template,
        vars,
        yes,
      } => self.handle_init(path, runtime, template, vars, yes).await,
      Commands::Start => self.handle_start().await,
      Commands::Stop => self.handle_stop().await,
      Commands::Proxy => self.handle_proxy().await,
      Commands::Bundle => self.handle_bundle().await,
      Commands::Create { template } => self.handle_create_template(template).await,
      Commands::Templates { command } => self.handle_templates(command).await,
      Commands::List { runtime } => self.handle_list(runtime).await,
      Commands::Cache { command } => self.handle_cache(command).await,
      Commands::Completions { shell } => {
        self.handle_completions(shell);
        Ok(())
      }
    }
  }

  async fn handle_init(
    &self,
    path: PathBuf,
    runtime_spec: Option<String>,
    template: Option<String>,
    vars: Vec<(String, String)>,
    skip_prompts: bool,
  ) -> Result<()> {
    use inquire::{Select, Text};

    // Interactive mode if no runtime specified and not using --yes flag
    let (runtime_spec, template, path) =
      if runtime_spec.is_none() && !skip_prompts {
        println!("{}", "🏗️  Create a new Realm environment".cyan().bold());
        println!();

        // Prompt for project name/path
        let project_name = Text::new("Project name:")
          .with_default(".venv")
          .with_help_message("Directory for the realm environment")
          .prompt()
          .map_err(|e| {
            RealmError::RuntimeError(crate::errors::RuntimeError::InstallationFailed(format!(
              "Prompt cancelled: {}",
              e
            )))
          })?;

        // Prompt for runtime
        let runtime_choice = Select::new(
          "Select runtime:",
          vec![
            "Bun (latest)",
            "Node.js (latest)",
            "Python (latest)",
            "Bun (specific version)",
            "Node.js (specific version)",
            "Python (specific version)",
          ],
        )
        .with_help_message("Choose your JavaScript/TypeScript runtime")
        .prompt()
        .map_err(|e| {
          RealmError::RuntimeError(crate::errors::RuntimeError::InstallationFailed(format!(
            "Prompt cancelled: {}",
            e
          )))
        })?;

        let runtime_spec =
          match runtime_choice {
            "Bun (latest)" => "bun".to_string(),
            "Node.js (latest)" => "node".to_string(),
            "Python (latest)" => "python".to_string(),
            "Bun (specific version)" => {
              let version = Text::new("Bun version:")
                .with_default("latest")
                .with_help_message("e.g., 1.0.1 or latest")
                .prompt()
                .map_err(|e| {
                  RealmError::RuntimeError(crate::errors::RuntimeError::InstallationFailed(
                    format!("Prompt cancelled: {}", e),
                  ))
                })?;
              format!("bun@{}", version)
            }
            "Node.js (specific version)" => {
              let version = Text::new("Node.js version:")
                .with_default("latest")
                .with_help_message("e.g., 20, 20.5.0, or latest")
                .prompt()
                .map_err(|e| {
                  RealmError::RuntimeError(crate::errors::RuntimeError::InstallationFailed(
                    format!("Prompt cancelled: {}", e),
                  ))
                })?;
              format!("node@{}", version)
            }
            "Python (specific version)" => {
              let version = Text::new("Python version:")
                .with_default("latest")
                .with_help_message("e.g., 3.12, 3.12.6, or latest")
                .prompt()
                .map_err(|e| {
                  RealmError::RuntimeError(crate::errors::RuntimeError::InstallationFailed(
                    format!("Prompt cancelled: {}", e),
                  ))
                })?;
              format!("python@{}", version)
            }
            _ => "bun".to_string(),
          };

        // Prompt for template
        let use_template = Select::new(
          "Use a project template?",
          vec![
            "No template",
            "React + Express",
            "React + FastAPI",
            "Vue + Express",
            "Svelte + Fastify",
            "Next.js",
          ],
        )
        .with_help_message("Start with a pre-configured project structure")
        .prompt()
        .map_err(|e| {
          RealmError::RuntimeError(crate::errors::RuntimeError::InstallationFailed(format!(
            "Prompt cancelled: {}",
            e
          )))
        })?;

        let template = match use_template {
          "No template" => None,
          "React + Express" => Some("react-express".to_string()),
          "React + FastAPI" => Some("react-fastapi".to_string()),
          "Vue + Express" => Some("vue-express".to_string()),
          "Svelte + Fastify" => Some("svelte-fastify".to_string()),
          "Next.js" => Some("nextjs".to_string()),
          _ => None,
        };

        println!();
        (runtime_spec, template, PathBuf::from(project_name))
      } else {
        // Non-interactive mode: use provided values or defaults
        let runtime_spec = runtime_spec.unwrap_or_else(|| "bun".to_string());
        (runtime_spec, template, path)
      };

    println!("{}", "🏗️  Initializing realm environment...".cyan().bold());

    // Parse runtime specification
    let mut runtime = Runtime::parse(&runtime_spec)?;

    // Check if we can use system-installed runtime
    // Only use system runtime if user requested "latest" or didn't specify version
    let use_system =
      runtime.version() == "latest" && self.runtime_manager.is_available_on_system(&runtime);

    if use_system {
      println!(
        "{} Using system-installed {} (found in PATH)",
        "✓".green().bold(),
        runtime.name()
      );
    } else {
      // Resolve "latest" to actual version if needed for .realm installation
      if runtime.version() == "latest" {
        runtime = self
          .runtime_manager
          .resolve_latest_to_actual(&runtime)
          .await?;
      }

      // Install runtime if not already installed in ~/.realm
      if !self.runtime_manager.is_version_installed(&runtime) {
        println!(
          "{} Getting {} {}...",
          "📦".cyan(),
          runtime.name(),
          runtime.version()
        );
        self.runtime_manager.install_version(&runtime).await?;
      } else {
        println!(
          "{} {} {} already installed",
          "✓".green().bold(),
          runtime.name(),
          runtime.version()
        );
      }
    }

    // Create project from template if specified
    if let Some(template_name) = &template {
      let project_dir = std::env::current_dir()?.join("project");
      println!(
        "{} Creating project from template '{}'...",
        "🎯".cyan(),
        template_name.bright_white()
      );

      // Convert vars Vec to HashMap
      let vars_map: HashMap<String, String> = vars.into_iter().collect();

      self.template_manager.init_from_template(
        template_name,
        &project_dir,
        vars_map,
        skip_prompts,
      )?;
      std::env::set_current_dir(&project_dir)?;
    }

    // Initialize realm environment
    let realm_env = RealmEnvironment::init(&path)?;

    // Set up Python-specific isolation if using Python runtime
    realm_env.setup_python_isolation(&runtime, &self.runtime_manager)?;

    println!("\n{} Realm environment initialized!", "✓".green().bold());
    println!(
      "{} Runtime: {} {}",
      "🎯".cyan(),
      runtime.name().bright_white(),
      runtime.version()
    );
    if let Some(template_name) = template {
      println!("{} Template: {}", "📄".cyan(), template_name.bright_white());
    }
    println!();
    println!("{}:", "Next steps".bright_white().bold());
    println!(
      "  {}",
      format!("source {}/bin/activate", path.display()).bright_cyan()
    );
    println!("  {}", "realm start".bright_cyan());

    Ok(())
  }

  async fn handle_start(&self) -> Result<()> {
    // Check if we're in an activated realm environment
    if std::env::var("REALM_ENV").is_err() {
      return Err(RealmError::ValidationError(
        "Not in an activated realm environment. Run 'source .venv/bin/activate' first.".to_string(),
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

    println!(
      "{} Fetching available {} versions...",
      "📦".cyan(),
      runtime.name().bright_white()
    );

    let versions = self
      .runtime_manager
      .list_available_versions(&runtime)
      .await?;

    if versions.is_empty() {
      println!("   {}", "No versions found".yellow());
    } else {
      println!("\n   {}:", "Available versions".bright_white().bold());
      for version in versions {
        let installed = self
          .runtime_manager
          .is_version_installed(&Runtime::from_name_version(runtime.name(), &version));
        if installed {
          println!(
            "   {} {} {}",
            "✓".green(),
            version.bright_white(),
            "(installed)".green()
          );
        } else {
          println!("   • {}", version);
        }
      }
    }

    Ok(())
  }

  async fn handle_cache(&self, command: CacheCommands) -> Result<()> {
    match command {
      CacheCommands::Clear => {
        println!("{} Clearing runtime version cache...", "🗑️".cyan());

        let cache_manager = CacheManager::new()?;
        cache_manager.clear_all()?;

        println!("{} Cache cleared successfully", "✓".green().bold());
        Ok(())
      }
    }
  }

  fn handle_completions(&self, shell: Shell) {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    generate(shell, &mut cmd, name, &mut io::stdout());
  }
}
