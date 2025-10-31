use anyhow::{anyhow, Context, Result};
use dirs::home_dir;
use include_dir::{include_dir, Dir};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tera::{Context as TeraContext, Tera};

use super::manifest::TemplateManifest;

static BUILTIN_TEMPLATES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates");

pub struct TemplateManager {
  user_templates_dir: PathBuf,
}

impl TemplateManager {
  pub fn new() -> Result<Self> {
    let home = home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
    let user_templates_dir = home.join(".realm").join("templates");

    if !user_templates_dir.exists() {
      fs::create_dir_all(&user_templates_dir).context("Failed to create templates directory")?;
    }

    Ok(Self { user_templates_dir })
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

    // Save template to user templates directory
    let template_dir = self.user_templates_dir.join(name);

    if template_dir.exists() {
      return Err(anyhow!("Template '{}' already exists", name));
    }

    fs::create_dir_all(&template_dir)?;

    // Copy current directory to template directory (excluding common patterns)
    self.copy_dir_filtered(&current_dir, &template_dir)?;

    println!("Template '{name}' created successfully");
    println!("Template saved to: {}", template_dir.display());

    Ok(())
  }

  #[allow(clippy::only_used_in_recursion)]
  fn copy_dir_filtered(&self, source: &Path, dest: &Path) -> Result<()> {
    for entry in fs::read_dir(source)? {
      let entry = entry?;
      let source_path = entry.path();

      // Skip common ignore patterns
      if let Some(name) = source_path.file_name().and_then(|n| n.to_str()) {
        if name.starts_with('.') && name != ".env" && name != ".gitignore" {
          continue;
        }
        if matches!(
          name,
          "node_modules" | "target" | "dist" | "build" | ".git" | ".venv"
        ) {
          continue;
        }
      }

      let file_name = entry.file_name();
      let dest_path = dest.join(&file_name);

      if source_path.is_dir() {
        fs::create_dir_all(&dest_path)?;
        self.copy_dir_filtered(&source_path, &dest_path)?;
      } else {
        fs::copy(&source_path, &dest_path)?;
      }
    }

    Ok(())
  }

  pub fn init_from_template(
    &self,
    template_name: &str,
    target_dir: &Path,
    provided_vars: HashMap<String, String>,
    skip_prompts: bool,
  ) -> Result<()> {
    if target_dir.exists() && target_dir.read_dir()?.next().is_some() {
      return Err(anyhow!("Target directory is not empty"));
    }

    fs::create_dir_all(target_dir)?;

    // Load template manifest if it exists
    let manifest = self.load_template_manifest(template_name);

    // Collect all variables (provided + prompted + defaults)
    let variables = if let Some(ref manifest) = manifest {
      self.collect_template_variables(manifest, provided_vars, target_dir, skip_prompts)?
    } else {
      provided_vars
    };

    // Create template context
    let context = self.create_template_context(target_dir, &variables)?;

    // Try built-in templates first
    if let Some(template_dir) = BUILTIN_TEMPLATES.get_dir(template_name) {
      self.copy_embedded_template(template_dir, target_dir, &context)?;
    }
    // Try user templates
    else {
      let user_template_path = self.user_templates_dir.join(template_name);
      if user_template_path.exists() && user_template_path.is_dir() {
        self.copy_filesystem_template(&user_template_path, target_dir, &context)?;
      } else {
        return Err(anyhow!("Template '{}' not found", template_name));
      }
    }

    println!(
      "Project created from template '{}' in {}",
      template_name,
      target_dir.display()
    );

    Ok(())
  }

  fn load_template_manifest(&self, template_name: &str) -> Option<TemplateManifest> {
    // Try built-in templates first
    if let Some(template_dir) = BUILTIN_TEMPLATES.get_dir(template_name) {
      if let Some(manifest_file) = template_dir.get_file("template.yaml") {
        if let Some(content) = manifest_file.contents_utf8() {
          return TemplateManifest::from_yaml_str(content).ok();
        }
      }
      // Try .yml extension
      if let Some(manifest_file) = template_dir.get_file("template.yml") {
        if let Some(content) = manifest_file.contents_utf8() {
          return TemplateManifest::from_yaml_str(content).ok();
        }
      }
    }

    // Try user templates
    let user_template_path = self.user_templates_dir.join(template_name);
    if user_template_path.exists() {
      let yaml_path = user_template_path.join("template.yaml");
      if yaml_path.exists() {
        return TemplateManifest::from_file(&yaml_path).ok();
      }
      let yml_path = user_template_path.join("template.yml");
      if yml_path.exists() {
        return TemplateManifest::from_file(&yml_path).ok();
      }
    }

    None
  }

  fn collect_template_variables(
    &self,
    manifest: &TemplateManifest,
    mut provided_vars: HashMap<String, String>,
    target_dir: &Path,
    skip_prompts: bool,
  ) -> Result<HashMap<String, String>> {
    let mut variables = HashMap::new();

    // Add project_name default
    let project_name = target_dir
      .file_name()
      .and_then(|n| n.to_str())
      .unwrap_or("project")
      .to_string();

    for var in &manifest.variables {
      let value = if let Some(provided) = provided_vars.remove(&var.name) {
        // Use provided value from CLI
        provided
      } else if skip_prompts {
        // Use default if skipping prompts
        self.resolve_default(&var.default, &project_name)
      } else {
        // Prompt user
        use inquire::Text;
        let default = self.resolve_default(&var.default, &project_name);
        Text::new(&var.prompt)
          .with_default(&default)
          .prompt()
          .map_err(|e| anyhow!("Prompt cancelled: {}", e))?
      };

      variables.insert(var.name.clone(), value);
    }

    Ok(variables)
  }

  fn resolve_default(&self, default: &str, project_name: &str) -> String {
    default.replace("{{directory_name}}", project_name)
  }

  fn create_template_context(
    &self,
    target_dir: &Path,
    variables: &HashMap<String, String>,
  ) -> Result<TeraContext> {
    let mut context = TeraContext::new();

    // Add project_name if not in variables
    if !variables.contains_key("project_name") {
      let project_name = target_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");
      context.insert("project_name", project_name);
    }

    // Add all variables to context
    for (key, value) in variables {
      context.insert(key, value);
    }

    // Add backwards compatibility defaults
    if !variables.contains_key("author") {
      context.insert("author", "");
    }
    if !variables.contains_key("description") {
      context.insert("description", "");
    }

    Ok(context)
  }

  fn copy_embedded_template(
    &self,
    template_dir: &Dir,
    target_dir: &Path,
    context: &TeraContext,
  ) -> Result<()> {
    self.copy_embedded_dir_recursive(template_dir, target_dir, template_dir.path(), context)
  }

  fn copy_embedded_dir_recursive(
    &self,
    dir: &Dir,
    target_base: &Path,
    template_root: &std::path::Path,
    context: &TeraContext,
  ) -> Result<()> {
    for entry in dir.entries() {
      if let Some(file) = entry.as_file() {
        let relative_path = file
          .path()
          .strip_prefix(template_root)
          .unwrap_or(file.path());
        let file_path = target_base.join(relative_path);

        if let Some(parent) = file_path.parent() {
          fs::create_dir_all(parent)?;
        }

        // Process file content with Tera if it's a text file
        let content = self.process_template_content(file.contents(), context)?;
        fs::write(&file_path, content)?;

        // Preserve executable permissions if set
        #[cfg(unix)]
        {
          use std::os::unix::fs::PermissionsExt;
          if relative_path.to_string_lossy().starts_with("bin/") {
            if let Ok(metadata) = fs::metadata(&file_path) {
              let mut perms = metadata.permissions();
              perms.set_mode(0o755);
              let _ = fs::set_permissions(&file_path, perms);
            }
          }
        }
      } else if let Some(subdir) = entry.as_dir() {
        self.copy_embedded_dir_recursive(subdir, target_base, template_root, context)?;
      }
    }

    Ok(())
  }

  fn copy_filesystem_template(
    &self,
    template_dir: &Path,
    target_dir: &Path,
    context: &TeraContext,
  ) -> Result<()> {
    for entry in fs::read_dir(template_dir)? {
      let entry = entry?;
      let source_path = entry.path();
      let file_name = entry.file_name();
      let target_path = target_dir.join(&file_name);

      if source_path.is_dir() {
        fs::create_dir_all(&target_path)?;
        self.copy_filesystem_template(&source_path, &target_path, context)?;
      } else {
        // Read file content and process with Tera
        let content = fs::read(&source_path)?;
        let processed_content = self.process_template_content(&content, context)?;
        fs::write(&target_path, processed_content)?;
      }
    }

    Ok(())
  }

  fn process_template_content(&self, content: &[u8], context: &TeraContext) -> Result<Vec<u8>> {
    // Try to convert to UTF-8 string for template processing
    if let Ok(text) = std::str::from_utf8(content) {
      // Only process if it looks like it contains template variables
      if text.contains("{{") || text.contains("{%") {
        match Tera::one_off(text, context, false) {
          Ok(rendered) => return Ok(rendered.into_bytes()),
          Err(_) => {
            // If template rendering fails, return original content
            return Ok(content.to_vec());
          }
        }
      }
    }

    // Return original content if not UTF-8 or no template variables
    Ok(content.to_vec())
  }

  pub fn list_templates(&self) -> Result<Vec<String>> {
    let mut templates = Vec::new();

    // Add built-in templates
    for dir in BUILTIN_TEMPLATES.dirs() {
      if let Some(name) = dir.path().file_name().and_then(|n| n.to_str()) {
        templates.push(name.to_string());
      }
    }

    // Add user templates
    if self.user_templates_dir.exists() {
      for entry in fs::read_dir(&self.user_templates_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
          if let Some(name) = entry.file_name().to_str() {
            if !templates.contains(&name.to_string()) {
              templates.push(name.to_string());
            }
          }
        }
      }
    }

    templates.sort();
    Ok(templates)
  }
}

impl Default for TemplateManager {
  fn default() -> Self {
    Self::new().expect("Failed to create TemplateManager")
  }
}
