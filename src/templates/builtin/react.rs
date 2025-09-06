use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::config::{ProcessConfig, RealmConfig};
use crate::templates::template::{Template, TemplateFile};

pub fn create_template(templates_dir: &Path) -> Result<()> {
    let template_dir = templates_dir.join("react-express");
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
}"#.to_string(),
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
"#.to_string(),
            executable: false,
        },
        TemplateFile {
            path: "backend/package.json".to_string(),
            content: r#"{
  "name": "backend",
  "type": "module",
  "scripts": {
    "dev": "bun run --hot server.ts"
  },
  "dependencies": {
    "express": "^4.18.0"
  },
  "devDependencies": {
    "@types/express": "^4.17.0"
  }
}"#.to_string(),
            executable: false,
        },
        TemplateFile {
            path: "backend/server.ts".to_string(),
            content: r#"import express from 'express';

const app = express();
const PORT = 4001;

app.use(express.json());

app.get('/api/health', (req, res) => {
  res.json({ status: 'ok', timestamp: new Date().toISOString() });
});

app.listen(PORT, () => {
  console.log(`Server running on http://localhost:${PORT}`);
});
"#.to_string(),
            executable: false,
        },
    ];

    let realm_config = RealmConfig {
        env: HashMap::new(),
        env_file: Some(".env".to_string()),
        processes: {
            let mut processes = HashMap::new();
            processes.insert("frontend".to_string(), ProcessConfig {
                command: "bun run dev".to_string(),
                port: Some(4000),
                routes: vec!["/".to_string(), "/assets/*".to_string()],
                working_directory: Some("frontend".to_string()),
            });
            processes.insert("backend".to_string(), ProcessConfig {
                command: "bun run dev".to_string(),
                port: Some(4001),
                routes: vec!["/api/*".to_string()],
                working_directory: Some("backend".to_string()),
            });
            processes
        },
        proxy_port: 8000,
    };

    let template = Template {
        name: "react-express".to_string(),
        description: "React frontend with Express backend using Bun".to_string(),
        version: "1.0.0".to_string(),
        files,
        realm_config,
        variables: HashMap::new(),
    };

    let template_content = serde_yaml::to_string(&template)?;
    fs::write(template_dir.join("template.yml"), template_content)?;

    Ok(())
}