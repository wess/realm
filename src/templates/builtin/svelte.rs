use crate::config::{ProcessConfig, RealmConfig};
use crate::templates::template::{Template, TemplateFile};
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn create_template(templates_dir: &Path) -> Result<()> {
    let template_dir = templates_dir.join("svelte-fastify");
    if template_dir.exists() {
        return Ok(());
    }

    fs::create_dir_all(&template_dir)?;

    let files = vec![
        TemplateFile {
            path: "frontend/package.json".to_string(),
            content: r#"{
  "name": "svelte-frontend",
  "type": "module",
  "scripts": {
    "dev": "vite dev",
    "build": "vite build",
    "preview": "vite preview"
  },
  "dependencies": {
    "@sveltejs/adapter-auto": "^2.0.0",
    "@sveltejs/kit": "^1.20.4",
    "svelte": "^4.0.5"
  },
  "devDependencies": {
    "vite": "^4.4.2"
  }
}"#
            .to_string(),
            executable: false,
        },
        TemplateFile {
            path: "frontend/vite.config.js".to_string(),
            content: r#"import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],
  server: {
    port: 4000
  }
});
"#
            .to_string(),
            executable: false,
        },
        TemplateFile {
            path: "frontend/src/routes/+page.svelte".to_string(),
            content: r#"<script>
  import { onMount } from 'svelte';
  
  let message = 'Loading...';
  
  onMount(async () => {
    try {
      const response = await fetch('/api/health');
      const data = await response.json();
      message = `Connected to backend! Status: ${data.status}`;
    } catch (error) {
      message = 'Failed to connect to backend';
    }
  });
</script>

<h1>Welcome to SvelteKit + Fastify</h1>
<p>{message}</p>

<style>
  h1 {
    color: #ff3e00;
    font-family: 'Helvetica Neue', Arial, sans-serif;
  }
</style>
"#
            .to_string(),
            executable: false,
        },
        TemplateFile {
            path: "backend/package.json".to_string(),
            content: r#"{
  "name": "fastify-backend",
  "type": "module",
  "scripts": {
    "dev": "bun run --hot server.ts"
  },
  "dependencies": {
    "fastify": "^4.24.0",
    "@fastify/cors": "^8.4.0"
  },
  "devDependencies": {
    "@types/node": "^20.0.0"
  }
}"#
            .to_string(),
            executable: false,
        },
        TemplateFile {
            path: "backend/server.ts".to_string(),
            content: r#"import Fastify from 'fastify';
import cors from '@fastify/cors';

const fastify = Fastify({ logger: true });

// Register CORS
await fastify.register(cors, {
  origin: true
});

// Health check route
fastify.get('/api/health', async (request, reply) => {
  return { 
    status: 'ok', 
    timestamp: new Date().toISOString(),
    service: 'fastify-backend'
  };
});

// API routes
fastify.get('/api/users', async (request, reply) => {
  return [
    { id: 1, name: 'Alice' },
    { id: 2, name: 'Bob' }
  ];
});

// Start server
const start = async () => {
  try {
    await fastify.listen({ port: 4001, host: '0.0.0.0' });
    console.log('🚀 Fastify server running on http://localhost:4001');
  } catch (err) {
    fastify.log.error(err);
    process.exit(1);
  }
};

start();
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
                    routes: vec!["/".to_string(), "/app/*".to_string(), "/_app/*".to_string()],
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
        name: "svelte-fastify".to_string(),
        description: "SvelteKit frontend with Fastify backend using Bun".to_string(),
        version: "1.0.0".to_string(),
        files,
        realm_config,
        variables: HashMap::new(),
    };

    let template_content = serde_yaml::to_string(&template)?;
    fs::write(template_dir.join("template.yml"), template_content)?;

    Ok(())
}
