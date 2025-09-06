use crate::config::{ProcessConfig, RealmConfig};
use crate::templates::template::{Template, TemplateFile};
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn create_template(templates_dir: &Path) -> Result<()> {
  let template_dir = templates_dir.join("vue-express");
  if template_dir.exists() {
    return Ok(());
  }

  fs::create_dir_all(&template_dir)?;

  let files = vec![
    TemplateFile {
      path: "frontend/package.json".to_string(),
      content: r#"{
  "name": "vue-frontend",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  },
  "dependencies": {
    "vue": "^3.3.0"
  },
  "devDependencies": {
    "@vitejs/plugin-vue": "^4.4.0",
    "vite": "^4.4.0"
  }
}"#
        .to_string(),
      executable: false,
    },
    TemplateFile {
      path: "frontend/src/App.vue".to_string(),
      content: r#"<template>
  <div id="app">
    <h1>Vue + Express</h1>
    <p>{{ message }}</p>
    <button @click="fetchData">Fetch Backend Data</button>
    <ul v-if="users.length">
      <li v-for="user in users" :key="user.id">{{ user.name }}</li>
    </ul>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'

const message = ref('Loading...')
const users = ref([])

const fetchData = async () => {
  try {
    const response = await fetch('/api/users')
    users.value = await response.json()
  } catch (error) {
    console.error('Failed to fetch users:', error)
  }
}

onMounted(async () => {
  try {
    const response = await fetch('/api/health')
    const data = await response.json()
    message.value = `Connected to backend! Status: ${data.status}`
  } catch (error) {
    message.value = 'Failed to connect to backend'
  }
})
</script>

<style>
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  text-align: center;
  color: #2c3e50;
  margin-top: 60px;
}

h1 {
  color: #42b883;
}

button {
  background-color: #42b883;
  color: white;
  border: none;
  padding: 10px 20px;
  border-radius: 5px;
  cursor: pointer;
  margin: 10px;
}
</style>
"#
      .to_string(),
      executable: false,
    },
    TemplateFile {
      path: "backend/package.json".to_string(),
      content: r#"{
  "name": "express-backend",
  "type": "module",
  "scripts": {
    "dev": "bun run --hot server.js"
  },
  "dependencies": {
    "express": "^4.18.0",
    "cors": "^2.8.5"
  }
}"#
        .to_string(),
      executable: false,
    },
    TemplateFile {
      path: "backend/server.js".to_string(),
      content: r#"import express from 'express';
import cors from 'cors';

const app = express();
const PORT = 4001;

// Middleware
app.use(cors());
app.use(express.json());

// Health check route
app.get('/api/health', (req, res) => {
  res.json({ 
    status: 'ok', 
    timestamp: new Date().toISOString(),
    service: 'express-backend'
  });
});

// API routes
app.get('/api/users', (req, res) => {
  res.json([
    { id: 1, name: 'Alice Johnson' },
    { id: 2, name: 'Bob Smith' },
    { id: 3, name: 'Charlie Brown' }
  ]);
});

app.listen(PORT, () => {
  console.log(`🚀 Express server running on http://localhost:${PORT}`);
});
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
          routes: vec![
            "/".to_string(),
            "/src/*".to_string(),
            "/assets/*".to_string(),
          ],
          working_directory: Some("frontend".to_string()),
        },
      );
      processes.insert(
        "backend".to_string(),
        ProcessConfig {
          command: "bun run dev".to_string(),
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
    name: "vue-express".to_string(),
    description: "Vue 3 frontend with Express backend using Bun".to_string(),
    version: "1.0.0".to_string(),
    files,
    realm_config,
    variables: HashMap::new(),
  };

  let template_content = serde_yaml::to_string(&template)?;
  fs::write(template_dir.join("template.yml"), template_content)?;

  Ok(())
}
