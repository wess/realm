use crate::config::RealmConfig;
use crate::runtime::types::Runtime;
use crate::runtime::manager::RuntimeManager;
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub struct RealmEnvironment {
  pub path: PathBuf,
  pub config: RealmConfig,
}

impl RealmEnvironment {
  pub fn init<P: AsRef<Path>>(path: P) -> Result<Self> {
    let env_path = path.as_ref().to_path_buf();

    if env_path.exists() {
      return Err(anyhow!(
        "Realm environment already exists at {}",
        env_path.display()
      ));
    }

    // Create directory structure
    fs::create_dir_all(&env_path).context("Failed to create realm environment directory")?;
    fs::create_dir_all(env_path.join("bin")).context("Failed to create bin directory")?;
    fs::create_dir_all(env_path.join("bun")).context("Failed to create bun directory")?;
    fs::create_dir_all(env_path.join("config")).context("Failed to create config directory")?;
    fs::create_dir_all(env_path.join("logs")).context("Failed to create logs directory")?;

    // Create default realm.yml if it doesn't exist in current directory
    let realm_yml_path = std::env::current_dir()?.join("realm.yml");
    let config = if realm_yml_path.exists() {
      RealmConfig::load(&realm_yml_path)?
    } else {
      let default_config = RealmConfig::default();
      default_config.save(&realm_yml_path)?;
      default_config
    };

    let realm_env = Self {
      path: env_path.clone(),
      config,
    };

    // Generate activation script
    realm_env.generate_activation_script()?;

    println!("Realm environment created at {}", env_path.display());
    println!("To activate: source {}/bin/activate", env_path.display());

    Ok(realm_env)
  }

  pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
    let env_path = path.as_ref().to_path_buf();

    if !env_path.exists() {
      return Err(anyhow!(
        "Realm environment not found at {}",
        env_path.display()
      ));
    }

    // Look for realm.yml in current directory or parent directories
    let realm_yml_path = Self::find_realm_yml()?;
    let config = RealmConfig::load(&realm_yml_path)?;

    Ok(Self {
      path: env_path,
      config,
    })
  }

  fn find_realm_yml() -> Result<PathBuf> {
    let mut current_dir = std::env::current_dir()?;

    loop {
      let realm_yml = current_dir.join("realm.yml");
      if realm_yml.exists() {
        return Ok(realm_yml);
      }

      if let Some(parent) = current_dir.parent() {
        current_dir = parent.to_path_buf();
      } else {
        break;
      }
    }

    Err(anyhow!(
      "realm.yml not found in current directory or parent directories"
    ))
  }

  fn generate_activation_script(&self) -> Result<()> {
    // Check if this is a Python environment (has pyvenv.cfg)
    let is_python_env = self.path.join("pyvenv.cfg").exists();

    let python_section = if is_python_env {
      format!(
        r#"
# Python virtual environment support
VIRTUAL_ENV="{}"
export VIRTUAL_ENV

if [ -n "${{_OLD_PYTHONHOME:-}}" ] ; then
    PYTHONHOME="${{_OLD_REALM_PYTHONHOME:-}}"
    export PYTHONHOME
    unset _OLD_REALM_PYTHONHOME
else
    unset PYTHONHOME
fi
"#,
        self.path.display()
      )
    } else {
      String::new()
    };

    let python_deactivate = if is_python_env {
      r#"
    if [ -n "${VIRTUAL_ENV:-}" ] ; then
        unset VIRTUAL_ENV
    fi

    if [ -n "${_OLD_REALM_PYTHONHOME:-}" ] ; then
        PYTHONHOME="${_OLD_REALM_PYTHONHOME:-}"
        export PYTHONHOME
        unset _OLD_REALM_PYTHONHOME
    fi
"#
    } else {
      ""
    };

    let activate_script = format!(
      r#"#!/bin/bash
# This file must be used with "source bin/activate" *from bash*
# you cannot run it directly

if [ "${{BASH_SOURCE-}}" = "${{0}}" ]; then
    echo "You must source this script: source ${{BASH_SOURCE}}"
    exit 33
fi

deactivate () {{
    # Reset old environment variables
    if [ -n "${{_OLD_REALM_PATH:-}}" ] ; then
        PATH="${{_OLD_REALM_PATH:-}}"
        export PATH
        unset _OLD_REALM_PATH
    fi

    if [ -n "${{_OLD_REALM_PS1:-}}" ] ; then
        PS1="${{_OLD_REALM_PS1:-}}"
        export PS1
        unset _OLD_REALM_PS1
    fi
{}
    unset REALM_ENV
    if [ ! "${{1:-}}" = "nondestructive" ] ; then
        unset -f deactivate
    fi
}}

# Unset irrelevant variables
deactivate nondestructive

REALM_ENV="{}"
export REALM_ENV

_OLD_REALM_PATH="$PATH"
PATH="{}:$PATH"
export PATH
{}
if [ -z "${{REALM_DISABLE_PROMPT:-}}" ] ; then
    _OLD_REALM_PS1="${{PS1:-}}"
    PS1="(realm) ${{PS1:-}}"
    export PS1
fi

# Load environment variables from realm config
# This would be populated dynamically based on realm.yml

echo "Realm environment activated"
echo "Run 'realm start' to start your processes"
echo "Run 'realm proxy' to start the development proxy"
echo "Run 'deactivate' to exit the realm environment"
"#,
      python_deactivate,
      self.path.display(),
      self.path.join("bin").display(),
      python_section
    );

    let activate_path = self.path.join("bin").join("activate");
    fs::write(&activate_path, activate_script).context("Failed to write activation script")?;

    // Make executable
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      let mut perms = fs::metadata(&activate_path)?.permissions();
      perms.set_mode(0o755);
      fs::set_permissions(&activate_path, perms)?;
    }

    Ok(())
  }

  pub fn get_bun_path(&self) -> PathBuf {
    self.path.join("bun")
  }

  pub fn get_logs_path(&self) -> PathBuf {
    self.path.join("logs")
  }

  pub fn get_config_path(&self) -> PathBuf {
    self.path.join("config")
  }

  pub fn setup_python_isolation(&self, runtime: &Runtime, runtime_manager: &RuntimeManager) -> Result<()> {
    match runtime {
      Runtime::Python(version) => {
        let python_version_parts: Vec<&str> = version.split('.').collect();
        let python_minor_version = if python_version_parts.len() >= 2 {
          format!("{}.{}", python_version_parts[0], python_version_parts[1])
        } else {
          "3.12".to_string()
        };

        // Create per-project site-packages directory
        let site_packages_dir = self.path
          .join("lib")
          .join(format!("python{}", python_minor_version))
          .join("site-packages");
        fs::create_dir_all(&site_packages_dir)
          .context("Failed to create site-packages directory")?;

        // Create symlink to shared Python binary
        let shared_python = runtime_manager.get_runtime_path(runtime);
        if !shared_python.exists() {
          return Err(anyhow!(
            "Python binary not found at {}. Installation may have failed.",
            shared_python.display()
          ));
        }

        let local_python = self.path.join("bin").join("python");
        let local_python3 = self.path.join("bin").join("python3");

        #[cfg(unix)]
        {
          use std::os::unix::fs::symlink;
          if !local_python3.exists() {
            symlink(&shared_python, &local_python3)
              .with_context(|| format!("Failed to create python3 symlink from {} to {}. You may need appropriate permissions.", shared_python.display(), local_python3.display()))?;
          }
          if !local_python.exists() {
            symlink(&shared_python, &local_python)
              .with_context(|| format!("Failed to create python symlink from {} to {}", shared_python.display(), local_python.display()))?;
          }
        }

        #[cfg(windows)]
        {
          use std::os::windows::fs::symlink_file;
          if !local_python3.exists() {
            symlink_file(&shared_python, local_python3.with_extension("exe"))
              .context("Failed to create python3 symlink. You may need administrator privileges on Windows.")?;
          }
          if !local_python.exists() {
            symlink_file(&shared_python, local_python.with_extension("exe"))
              .context("Failed to create python symlink. You may need administrator privileges on Windows.")?;
          }
        }

        // Create pyvenv.cfg pointing to shared Python
        let pyvenv_cfg = format!(
          "home = {}\ninclude-system-site-packages = false\nversion = {}\n",
          runtime_manager
            .get_runtime_versions_dir(runtime)
            .join(version)
            .display(),
          version
        );

        fs::write(self.path.join("pyvenv.cfg"), pyvenv_cfg)
          .context("Failed to write pyvenv.cfg")?;

        // Create symlink to pip if it exists
        if let Some(pip_path) = runtime_manager.get_pip_path(runtime) {
          let local_pip = self.path.join("bin").join("pip");
          let local_pip3 = self.path.join("bin").join("pip3");

          #[cfg(unix)]
          {
            use std::os::unix::fs::symlink;
            if !local_pip3.exists() {
              if let Err(e) = symlink(&pip_path, &local_pip3) {
                eprintln!("Warning: Failed to create pip3 symlink: {}", e);
              }
            }
            if !local_pip.exists() {
              if let Err(e) = symlink(&pip_path, &local_pip) {
                eprintln!("Warning: Failed to create pip symlink: {}", e);
              }
            }
          }

          #[cfg(windows)]
          {
            use std::os::windows::fs::symlink_file;
            if !local_pip3.exists() {
              if let Err(e) = symlink_file(&pip_path, local_pip3.with_extension("exe")) {
                eprintln!("Warning: Failed to create pip3 symlink: {}", e);
              }
            }
            if !local_pip.exists() {
              if let Err(e) = symlink_file(&pip_path, local_pip.with_extension("exe")) {
                eprintln!("Warning: Failed to create pip symlink: {}", e);
              }
            }
          }
        } else {
          eprintln!("Warning: pip not found in Python installation. You may need to install it manually.");
        }

        println!("✨ Created per-project Python environment with isolated site-packages");

        // Regenerate activation script to include VIRTUAL_ENV
        self.generate_activation_script()?;
      }
      _ => {}
    }
    Ok(())
  }

  pub fn regenerate_activation_script(&self) -> Result<()> {
    self.generate_activation_script()
  }
}
