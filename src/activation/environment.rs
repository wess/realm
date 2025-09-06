use crate::config::RealmConfig;
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
      self.path.display(),
      self.path.join("bin").display()
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
}
