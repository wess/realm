use crate::config::{ProcessConfig, RealmConfig};
use crate::templates::template::{Template, TemplateFile};
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn create_template(templates_dir: &Path) -> Result<()> {
  let template_dir = templates_dir.join("react-fastapi");
  if template_dir.exists() {
    return Ok(());
  }

  fs::create_dir_all(&template_dir)?;

  let files = vec![
    TemplateFile {
      path: "frontend/package.json".to_string(),
      content: r#"{
  "name": "frontend",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build"
  },
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0"
  },
  "devDependencies": {
    "@vitejs/plugin-react": "^4.0.0",
    "vite": "^4.4.0"
  }
}"#
        .to_string(),
      executable: false,
    },
    TemplateFile {
      path: "frontend/vite.config.js".to_string(),
      content: r#"import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  server: {
    port: 4000
  }
})
"#
      .to_string(),
      executable: false,
    },
    TemplateFile {
      path: "backend/requirements.txt".to_string(),
      content: r#"fastapi==0.109.0
uvicorn[standard]==0.27.0
"#
        .to_string(),
      executable: false,
    },
    TemplateFile {
      path: "backend/main.py".to_string(),
      content: r#"from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from datetime import datetime

app = FastAPI()

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

@app.get("/api/health")
async def health():
    return {"status": "ok", "timestamp": datetime.now().isoformat()}

@app.get("/api/hello")
async def hello():
    return {"message": "Hello from FastAPI!", "timestamp": datetime.now().isoformat()}

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=4001)
"#
      .to_string(),
      executable: false,
    },
  ];

  let realm_config = RealmConfig {
    env: HashMap::new(),
    env_file: Some(".env".to_string()),
    processes: {
      let mut processes = HashMap::new();
      processes.insert(
        "frontend".to_string(),
        ProcessConfig {
          command: "bun run dev".to_string(),
          port: Some(4000),
          routes: vec!["/".to_string(), "/assets/*".to_string()],
          working_directory: Some("frontend".to_string()),
        },
      );
      processes.insert(
        "backend".to_string(),
        ProcessConfig {
          command: "uvicorn main:app --reload --port 4001".to_string(),
          port: Some(4001),
          routes: vec!["/api/*".to_string()],
          working_directory: Some("backend".to_string()),
        },
      );
      processes
    },
    proxy_port: 8000,
  };

  let template = Template {
    name: "react-fastapi".to_string(),
    description: "React frontend with FastAPI backend using Python".to_string(),
    version: "1.0.0".to_string(),
    files,
    realm_config,
    variables: HashMap::new(),
  };

  let template_content = serde_yaml::to_string(&template)?;
  fs::write(template_dir.join("template.yml"), template_content)?;

  Ok(())
}
