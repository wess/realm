use super::info::ProcessInfo;
use crate::config::RealmConfig;
use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

pub struct ProcessManager {
  processes: Arc<Mutex<HashMap<String, ProcessInfo>>>,
}

impl ProcessManager {
  pub fn new() -> Self {
    Self {
      processes: Arc::new(Mutex::new(HashMap::new())),
    }
  }

  pub fn load_processes(&self, config: &RealmConfig) -> Result<()> {
    let mut processes = self.processes.lock().unwrap();
    processes.clear();

    for (name, process_config) in &config.processes {
      let process_info = ProcessInfo {
        name: name.clone(),
        config: process_config.clone(),
        child: None,
        port: process_config.port,
      };
      processes.insert(name.clone(), process_info);
    }

    Ok(())
  }

  pub fn start_process(&self, name: &str) -> Result<()> {
    let mut processes = self.processes.lock().unwrap();
    let process_info = processes
      .get_mut(name)
      .ok_or_else(|| anyhow!("Process '{}' not found", name))?;

    if process_info.child.is_some() {
      return Ok(());
    }

    println!("Starting process: {name}");

    // Parse command and arguments
    let parts: Vec<&str> = process_info.config.command.split_whitespace().collect();
    if parts.is_empty() {
      return Err(anyhow!("Empty command for process '{}'", name));
    }

    let mut cmd = Command::new(parts[0]);
    if parts.len() > 1 {
      cmd.args(&parts[1..]);
    }

    // Set working directory if specified
    if let Some(working_dir) = &process_info.config.working_directory {
      cmd.current_dir(working_dir);
    }

    // Set up stdio
    cmd
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .stdin(Stdio::null());

    let child = cmd
      .spawn()
      .context(format!("Failed to start process '{name}'"))?;
    process_info.child = Some(child);

    println!("Process '{name}' started successfully");
    Ok(())
  }

  pub fn stop_process(&self, name: &str) -> Result<()> {
    let mut processes = self.processes.lock().unwrap();
    let process_info = processes
      .get_mut(name)
      .ok_or_else(|| anyhow!("Process '{}' not found", name))?;

    if let Some(mut child) = process_info.child.take() {
      println!("Stopping process: {name}");

      // Try graceful termination first
      let _ = child.kill();
      let _ = child.wait();

      println!("Process '{name}' stopped");
    }

    Ok(())
  }

  pub fn restart_process(&self, name: &str) -> Result<()> {
    self.stop_process(name)?;
    self.start_process(name)
  }

  pub fn start_all(&self) -> Result<()> {
    let process_names: Vec<String> = {
      let processes = self.processes.lock().unwrap();
      processes.keys().cloned().collect()
    };

    for name in process_names {
      if let Err(e) = self.start_process(&name) {
        eprintln!("Failed to start process '{name}': {e}");
      }
    }

    Ok(())
  }

  pub fn stop_all(&self) -> Result<()> {
    let process_names: Vec<String> = {
      let processes = self.processes.lock().unwrap();
      processes.keys().cloned().collect()
    };

    for name in process_names {
      if let Err(e) = self.stop_process(&name) {
        eprintln!("Failed to stop process '{name}': {e}");
      }
    }

    Ok(())
  }

  pub fn is_running(&self, name: &str) -> bool {
    let processes = self.processes.lock().unwrap();
    if let Some(process_info) = processes.get(name) {
      if let Some(child) = &process_info.child {
        // Check if process is still alive
        !matches!(child.id(), 0)
      } else {
        false
      }
    } else {
      false
    }
  }

  pub fn get_process_port(&self, name: &str) -> Option<u16> {
    let processes = self.processes.lock().unwrap();
    processes.get(name).and_then(|p| p.port)
  }

  pub fn get_process_routes(&self, name: &str) -> Vec<String> {
    let processes = self.processes.lock().unwrap();
    processes
      .get(name)
      .map(|p| p.config.routes.clone())
      .unwrap_or_default()
  }

  pub fn list_processes(&self) -> Vec<String> {
    let processes = self.processes.lock().unwrap();
    processes.keys().cloned().collect()
  }
}

impl Default for ProcessManager {
  fn default() -> Self {
    Self::new()
  }
}

// Implement Clone for ProcessManager
impl Clone for ProcessManager {
  fn clone(&self) -> Self {
    Self {
      processes: Arc::clone(&self.processes),
    }
  }
}
