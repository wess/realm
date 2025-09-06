use super::builtin::{nextjs, react, svelte, vue};
use super::template::{Template, TemplateFile};
use crate::config::RealmConfig;
use anyhow::{anyhow, Context, Result};
use dirs::home_dir;
use std::fs;
use std::path::{Path, PathBuf};

pub struct TemplateManager {
  templates_dir: PathBuf,
}

impl TemplateManager {
  pub fn new() -> Result<Self> {
    let home = home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
    let templates_dir = home.join(".realm").join("templates");

    if !templates_dir.exists() {
      fs::create_dir_all(&templates_dir).context("Failed to create templates directory")?;
    }

    Ok(Self { templates_dir })
  }

  pub fn create_template_from_current_dir(&self, name: &str) -> Result<()> {
    let current_dir = std::env::current_dir()?;

    // Check if realm.yml exists
    let realm_yml_path = current_dir.join("realm.yml");
    if !realm_yml_path.exists() {
      return Err(anyhow!(
        "No realm.yml found in current directory. Initialize a realm project first."
      ));
    }

    let realm_config = RealmConfig::load(&realm_yml_path)?;

    // Collect all files in current directory (excluding common ignore patterns)
    let mut files = Vec::new();
    self.collect_template_files(&current_dir, &current_dir, &mut files)?;

    let template = Template {
      name: name.to_string(),
      description: format!("Template created from {}", current_dir.display()),
      version: "1.0.0".to_string(),
      files,
      realm_config,
      variables: std::collections::HashMap::new(),
    };

    // Save template
    let template_dir = self.templates_dir.join(name);
    fs::create_dir_all(&template_dir)?;

    let template_file = template_dir.join("template.yml");
    let template_content = serde_yaml::to_string(&template)?;
    fs::write(template_file, template_content)?;

    println!("Template '{name}' created successfully");
    println!("Template saved to: {}", template_dir.display());

    Ok(())
  }

  pub fn init_from_template(&self, template_name: &str, target_dir: &Path) -> Result<()> {
    let template = self.load_template(template_name)?;

    if target_dir.exists() && target_dir.read_dir()?.next().is_some() {
      return Err(anyhow!("Target directory is not empty"));
    }

    fs::create_dir_all(target_dir)?;

    // Create files from template
    for file in &template.files {
      let file_path = target_dir.join(&file.path);

      if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
      }

      // Process template variables (simple string replacement)
      let content = self.process_template_variables(&file.content, &template.variables);
      fs::write(&file_path, content)?;

      // Set executable if needed
      #[cfg(unix)]
      if file.executable {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&file_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&file_path, perms)?;
      }
    }

    // Create realm.yml
    let realm_yml_path = target_dir.join("realm.yml");
    template.realm_config.save(&realm_yml_path)?;

    println!(
      "Project created from template '{}' in {}",
      template_name,
      target_dir.display()
    );
    println!("Next steps:");
    println!("  cd {}", target_dir.display());
    println!("  realm init .venv");
    println!("  source .venv/bin/activate");
    println!("  realm start");

    Ok(())
  }

  pub fn list_templates(&self) -> Result<Vec<String>> {
    let mut templates = Vec::new();

    if !self.templates_dir.exists() {
      return Ok(templates);
    }

    for entry in fs::read_dir(&self.templates_dir)? {
      let entry = entry?;
      if entry.file_type()?.is_dir() {
        if let Some(name) = entry.file_name().to_str() {
          templates.push(name.to_string());
        }
      }
    }

    templates.sort();
    Ok(templates)
  }

  fn load_template(&self, name: &str) -> Result<Template> {
    let template_file = self.templates_dir.join(name).join("template.yml");

    if !template_file.exists() {
      return Err(anyhow!("Template '{}' not found", name));
    }

    let content = fs::read_to_string(template_file)?;
    let template: Template = serde_yaml::from_str(&content)?;
    Ok(template)
  }

  fn collect_template_files(
    &self,
    base_dir: &Path,
    current_dir: &Path,
    files: &mut Vec<TemplateFile>,
  ) -> Result<()> {
    for entry in fs::read_dir(current_dir)? {
      let entry = entry?;
      let path = entry.path();

      // Skip common ignore patterns
      if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if name.starts_with('.') && name != ".env" {
          continue;
        }
        if matches!(name, "node_modules" | "target" | "dist" | "build" | ".git") {
          continue;
        }
      }

      let relative_path = path.strip_prefix(base_dir)?.to_string_lossy().to_string();

      if path.is_file() {
        let content = fs::read_to_string(&path)?;
        let executable = self.is_executable(&path)?;

        files.push(TemplateFile {
          path: relative_path,
          content,
          executable,
        });
      } else if path.is_dir() {
        self.collect_template_files(base_dir, &path, files)?;
      }
    }

    Ok(())
  }

  fn is_executable(&self, path: &Path) -> Result<bool> {
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      let metadata = fs::metadata(path)?;
      Ok(metadata.permissions().mode() & 0o111 != 0)
    }
    #[cfg(not(unix))]
    {
      Ok(false)
    }
  }

  fn process_template_variables(
    &self,
    content: &str,
    _variables: &std::collections::HashMap<String, String>,
  ) -> String {
    // Simple implementation - could be enhanced with proper templating
    content.to_string()
  }

  pub fn create_builtin_templates(&self) -> Result<()> {
    react::create_template(&self.templates_dir)?;
    svelte::create_template(&self.templates_dir)?;
    vue::create_template(&self.templates_dir)?;
    nextjs::create_template(&self.templates_dir)?;
    Ok(())
  }
}

impl Default for TemplateManager {
  fn default() -> Self {
    Self::new().expect("Failed to create TemplateManager")
  }
}
