use crate::config::{ProcessConfig, RealmConfig};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct Bundler {
    config: RealmConfig,
    project_root: PathBuf,
    dist_dir: PathBuf,
}

impl Bundler {
    pub fn new(config: RealmConfig) -> Result<Self> {
        let project_root = std::env::current_dir()?;
        let dist_dir = project_root.join("dist");

        Ok(Self {
            config,
            project_root,
            dist_dir,
        })
    }

    pub fn bundle(&self) -> Result<()> {
        println!("Creating deployment bundle...");

        // Clean and create dist directory
        if self.dist_dir.exists() {
            fs::remove_dir_all(&self.dist_dir)?;
        }
        fs::create_dir_all(&self.dist_dir)?;

        // Build all processes
        self.build_processes()?;

        // Generate Docker artifacts
        self.generate_dockerfile()?;
        self.generate_docker_compose()?;
        self.generate_nginx_config()?;

        // Copy built assets
        self.copy_built_assets()?;

        // Generate deployment scripts
        self.generate_deployment_scripts()?;

        println!("✓ Bundle created successfully in ./dist/");
        println!("✓ Ready to deploy with: cd dist && docker-compose up");

        Ok(())
    }

    fn build_processes(&self) -> Result<()> {
        println!("Building processes...");

        for (name, process_config) in &self.config.processes {
            println!("  Building process: {}", name);
            self.build_process(name, process_config)?;
        }

        Ok(())
    }

    fn build_process(&self, name: &str, config: &ProcessConfig) -> Result<()> {
        let working_dir = if let Some(wd) = &config.working_directory {
            self.project_root.join(wd)
        } else {
            self.project_root.clone()
        };

        // Determine build command based on process type
        if working_dir.join("package.json").exists() {
            // Node.js/Bun project
            self.build_nodejs_process(name, &working_dir)?;
        } else if working_dir.join("Cargo.toml").exists() {
            // Rust project
            self.build_rust_process(name, &working_dir)?;
        } else if working_dir.join("requirements.txt").exists()
            || working_dir.join("pyproject.toml").exists()
        {
            // Python project
            self.build_python_process(name, &working_dir)?;
        } else {
            println!("    Unknown project type, copying source files...");
            self.copy_source_files(name, &working_dir)?;
        }

        Ok(())
    }

    fn build_nodejs_process(&self, name: &str, working_dir: &Path) -> Result<()> {
        // Install dependencies
        let output = Command::new("bun")
            .args(["install"])
            .current_dir(working_dir)
            .output()
            .context("Failed to run bun install")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "bun install failed for {}: {}",
                name,
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Check if there's a build script
        let package_json = fs::read_to_string(working_dir.join("package.json"))?;
        if package_json.contains("\"build\"") {
            let output = Command::new("bun")
                .args(["run", "build"])
                .current_dir(working_dir)
                .output()
                .context("Failed to run build command")?;

            if !output.status.success() {
                return Err(anyhow::anyhow!(
                    "Build failed for {}: {}",
                    name,
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }

        // Copy built files to dist
        let process_dist = self.dist_dir.join(name);
        fs::create_dir_all(&process_dist)?;

        // Copy source and built files
        Self::copy_dir_contents(working_dir, &process_dist)?;

        Ok(())
    }

    fn build_rust_process(&self, name: &str, working_dir: &Path) -> Result<()> {
        let output = Command::new("cargo")
            .args(["build", "--release"])
            .current_dir(working_dir)
            .output()
            .context("Failed to run cargo build")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Cargo build failed for {}: {}",
                name,
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let process_dist = self.dist_dir.join(name);
        fs::create_dir_all(&process_dist)?;

        // Copy target/release binary
        let target_dir = working_dir.join("target").join("release");
        Self::copy_dir_contents(&target_dir, &process_dist)?;

        Ok(())
    }

    fn build_python_process(&self, name: &str, working_dir: &Path) -> Result<()> {
        let process_dist = self.dist_dir.join(name);
        fs::create_dir_all(&process_dist)?;

        // Just copy Python source files - containerization will handle dependencies
        Self::copy_dir_contents(working_dir, &process_dist)?;

        Ok(())
    }

    fn copy_source_files(&self, name: &str, working_dir: &Path) -> Result<()> {
        let process_dist = self.dist_dir.join(name);
        fs::create_dir_all(&process_dist)?;
        Self::copy_dir_contents(working_dir, &process_dist)?;
        Ok(())
    }

    fn copy_dir_contents(from: &Path, to: &Path) -> Result<()> {
        if !from.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(from)? {
            let entry = entry?;
            let path = entry.path();
            let name = entry.file_name();

            // Skip common ignore patterns
            if let Some(name_str) = name.to_str() {
                if matches!(
                    name_str,
                    "node_modules" | "target" | ".git" | "dist" | "build" | ".realm"
                ) {
                    continue;
                }
            }

            let dest_path = to.join(&name);

            if path.is_dir() {
                fs::create_dir_all(&dest_path)?;
                Self::copy_dir_contents(&path, &dest_path)?;
            } else {
                fs::copy(&path, &dest_path)?;
            }
        }

        Ok(())
    }

    fn copy_built_assets(&self) -> Result<()> {
        // Copy any additional assets like static files, configs, etc.
        if self.project_root.join(".env").exists() {
            fs::copy(
                self.project_root.join(".env"),
                self.dist_dir.join(".env.example"),
            )?;
        }

        Ok(())
    }

    fn generate_dockerfile(&self) -> Result<()> {
        let dockerfile_content = format!(
            r#"# Multi-stage Dockerfile generated by Realm
FROM node:18-alpine as base

# Install Bun
RUN npm install -g bun

# Create app directory
WORKDIR /app

# Copy all built processes
{}

# Expose proxy port
EXPOSE {}

# Start command will be overridden by docker-compose
CMD ["echo", "Use docker-compose to start services"]
"#,
            self.generate_dockerfile_copy_commands(),
            self.config.proxy_port
        );

        fs::write(self.dist_dir.join("Dockerfile"), dockerfile_content)?;
        Ok(())
    }

    fn generate_dockerfile_copy_commands(&self) -> String {
        let mut commands = String::new();

        for name in self.config.processes.keys() {
            commands.push_str(&format!("COPY ./{} /app/{}\n", name, name));
        }

        commands
    }

    fn generate_docker_compose(&self) -> Result<()> {
        let mut services = String::new();

        // Generate service for each process
        for (name, config) in &self.config.processes {
            let port = config.port.unwrap_or(3000);
            let working_dir = config
                .working_directory
                .clone()
                .unwrap_or_else(|| name.clone());

            services.push_str(&format!(
                r#"  {}:
    build: .
    working_dir: /app/{}
    command: {}
    ports:
      - "{}:{}"
    environment:
{}
    networks:
      - realm-network

"#,
                name,
                working_dir,
                config.command,
                port,
                port,
                self.generate_env_vars()
            ));
        }

        // Add nginx service
        services.push_str(&format!(
            r#"  nginx:
    image: nginx:alpine
    ports:
      - "{}:{}"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
    depends_on:
{}
    networks:
      - realm-network

"#,
            self.config.proxy_port,
            self.config.proxy_port,
            self.generate_nginx_depends_on()
        ));

        let docker_compose_content = format!(
            r#"version: '3.8'

services:
{}

networks:
  realm-network:
    driver: bridge
"#,
            services
        );

        fs::write(
            self.dist_dir.join("docker-compose.yml"),
            docker_compose_content,
        )?;
        Ok(())
    }

    fn generate_env_vars(&self) -> String {
        let mut env_vars = String::new();

        for (key, value) in &self.config.env {
            env_vars.push_str(&format!("      - {}={}\n", key, value));
        }

        env_vars
    }

    fn generate_nginx_depends_on(&self) -> String {
        self.config
            .processes
            .keys()
            .map(|name| format!("      - {}", name))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_nginx_config(&self) -> Result<()> {
        let mut upstream_servers = String::new();
        let mut location_blocks = String::new();

        for (name, config) in &self.config.processes {
            let port = config.port.unwrap_or(3000);

            // Create upstream
            upstream_servers.push_str(&format!(
                r#"
    upstream {} {{
        server {}:{};
    }}
"#,
                name, name, port
            ));

            // Create location blocks for routes
            for route in &config.routes {
                let location = if route == "/" {
                    "/".to_string()
                } else {
                    route.replace("*", "").to_string()
                };

                location_blocks.push_str(&format!(
                    r#"
        location {} {{
            proxy_pass http://{};
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            
            # WebSocket support
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
        }}
"#,
                    location, name
                ));
            }
        }

        let nginx_config = format!(
            r#"events {{
    worker_connections 1024;
}}

http {{
    sendfile on;
    tcp_nopush on;
    tcp_nodelay on;
    keepalive_timeout 65;
    types_hash_max_size 2048;

    include /etc/nginx/mime.types;
    default_type application/octet-stream;

    # Gzip compression
    gzip on;
    gzip_vary on;
    gzip_min_length 1024;
    gzip_types text/plain text/css text/xml text/javascript application/javascript application/xml+rss application/json;

{}

    server {{
        listen {};
        server_name localhost;

        # Security headers
        add_header X-Frame-Options "SAMEORIGIN" always;
        add_header X-Content-Type-Options "nosniff" always;
        add_header X-XSS-Protection "1; mode=block" always;

{}

        # Health check endpoint
        location /health {{
            access_log off;
            return 200 "healthy\n";
            add_header Content-Type text/plain;
        }}
    }}
}}
"#,
            upstream_servers, self.config.proxy_port, location_blocks
        );

        fs::write(self.dist_dir.join("nginx.conf"), nginx_config)?;
        Ok(())
    }

    fn generate_deployment_scripts(&self) -> Result<()> {
        // Deploy script
        let deploy_script = r#"#!/bin/bash
set -e

echo "🚀 Deploying Realm application..."

# Build and start services
docker-compose build
docker-compose up -d

echo "✅ Deployment complete!"
echo "🌐 Application available at: http://localhost:8000"
echo "📊 View logs: docker-compose logs -f"
echo "🛑 Stop services: docker-compose down"
"#;

        fs::write(self.dist_dir.join("deploy.sh"), deploy_script)?;

        // Make executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(self.dist_dir.join("deploy.sh"))?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(self.dist_dir.join("deploy.sh"), perms)?;
        }

        // README for deployment
        let readme_content = format!(
            r#"# Realm Deployment Bundle

This directory contains everything needed to deploy your Realm application.

## Quick Start

```bash
# Deploy the application
./deploy.sh

# Or manually:
docker-compose up -d
```

## Services

Your application includes the following services:
{}

## Configuration

- **Proxy Port**: {} (configured in docker-compose.yml)
- **Environment Variables**: Defined in docker-compose.yml
- **Nginx Config**: Custom routing in nginx.conf

## Commands

- `docker-compose up -d` - Start services in background
- `docker-compose logs -f` - View logs
- `docker-compose down` - Stop services
- `docker-compose build` - Rebuild images

## Production Notes

1. Update environment variables in docker-compose.yml
2. Configure SSL/TLS termination (not included)
3. Set up proper logging and monitoring
4. Consider using Docker Swarm or Kubernetes for scaling
"#,
            self.generate_services_list(),
            self.config.proxy_port
        );

        fs::write(self.dist_dir.join("README.md"), readme_content)?;

        Ok(())
    }

    fn generate_services_list(&self) -> String {
        self.config
            .processes
            .iter()
            .map(|(name, config)| {
                let port = config.port.unwrap_or(3000);
                let routes = config.routes.join(", ");
                format!("- **{}**: Port {} (Routes: {})", name, port, routes)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
