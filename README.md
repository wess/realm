# Realm 🏰

[![CI](https://github.com/wess/realm/workflows/CI/badge.svg)](https://github.com/wess/realm/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/realmenv.svg)](https://crates.io/crates/realmenv)
[![GitHub release](https://img.shields.io/github/release/wess/realm.svg)](https://github.com/wess/realm/releases)

## What is Realm?

Realm eliminates the complexity of modern full-stack development by providing:

- **Virtualenv-like environments** for complete project isolation
- **Built-in process manager** (like Foreman) with intelligent routing
- **Multi-runtime support** (Bun, Node.js, Python) with automatic version management
- **Zero-config proxy server** that routes requests to your services
- **Project templates** to eliminate boilerplate
- **One-command deployment** with Docker generation

## Installation

### From crates.io (Recommended)
```bash
# Install using cargo
cargo install realmenv
```

### From Source
```bash
# Clone and build from source
git clone https://github.com/wess/realm
cd realm
cargo install --path .
```

### Pre-built Binaries
Download the latest release for your platform:

- **macOS (Intel)**: [realm-macos-amd64](https://github.com/wess/realm/releases/latest/download/realm-macos-amd64)
- **macOS (Apple Silicon)**: [realm-macos-arm64](https://github.com/wess/realm/releases/latest/download/realm-macos-arm64)
- **Linux (x64)**: [realm-linux-amd64](https://github.com/wess/realm/releases/latest/download/realm-linux-amd64)
- **Linux (ARM64)**: [realm-linux-arm64](https://github.com/wess/realm/releases/latest/download/realm-linux-arm64)
- **Windows**: [realm-windows-amd64.exe](https://github.com/wess/realm/releases/latest/download/realm-windows-amd64.exe)

### Quick Install Script (macOS/Linux)
```bash
curl -sSfL https://github.com/wess/realm/releases/latest/download/install.sh | bash
```

This will download and install realm to `/usr/local/bin`.

## Quick Start

### Interactive Mode (Recommended for New Users)
```bash
# Just run init without flags - Realm will guide you!
realm init

# Follow the prompts:
# → Project name: myapp
# → Select runtime: Bun (latest)
# → Use a template?: React + Express
# → Done!

cd myapp
source .venv/bin/activate
realm start
# Visit http://localhost:8000 - it just works!
```

### Command-Line Mode (For Scripts/CI)
```bash
# Create a new full-stack project with flags
realm init myapp --runtime=bun --template=react-express

# Or use defaults quickly
realm init -y

# Activate the environment
cd myapp
source .venv/bin/activate

# Start everything (processes + proxy)
realm start
```

### Python Stack (FastAPI)
```bash
# Create a FastAPI + React project
realm init myapp --template=react-fastapi --runtime=python@3.12

# Activate the environment
cd myapp
source .venv/bin/activate

# Install Python dependencies
pip install -r backend/requirements.txt

# Start everything (processes + proxy)
realm start
# Visit http://localhost:8000 - it just works!
```

## Core Workflow

### 1. Initialize Environment

**Interactive Mode** (prompts you for choices):
```bash
realm init
# → Asks for project name
# → Asks which runtime (Bun/Node/Python)
# → Asks which template (or none)
```

**Command-Line Mode** (for automation):
```bash
# With specific options
realm init .venv --runtime=node@20 --template=vue-express

# Quick defaults (skips prompts)
realm init -y

# Empty environment
realm init .venv --runtime=bun
```

### 2. Activate Environment
```bash
source .venv/bin/activate
# Your shell now shows: (realm) $
```

### 3. List Available Versions
```bash
# List available Python versions (shows cache status)
realm list --runtime=python

# List available Node versions
realm list --runtime=node

# List available Bun versions
realm list --runtime=bun

# Clear cached version lists (24hr cache)
realm cache clear
```

### Generate Shell Completions
```bash
# Bash
realm completions bash > /etc/bash_completion.d/realm

# Zsh
realm completions zsh > ~/.zfunc/_realm

# Fish
realm completions fish > ~/.config/fish/completions/realm.fish
```

### 4. Start Development
```bash
# Start all processes + proxy server
realm start

# Or start components separately
realm proxy     # Just the proxy
realm stop      # Stop everything
```

### 5. Deploy
```bash
# Generate Docker deployment artifacts
realm bundle
cd dist && ./deploy.sh
```

## Configuration

Realm uses `realm.yml` for project configuration:

```yaml
proxy_port: 8000

env:
  NODE_ENV: development
  API_URL: http://localhost:4001

env_file: .env

processes:
  frontend:
    command: "bun run dev"
    port: 4000
    routes: ["/", "/assets/*"]
    working_directory: "frontend"
    
  backend:
    command: "bun run server"  
    port: 4001
    routes: ["/api/*", "/health"]
    working_directory: "backend"
```

## Templates

Realm includes built-in templates for common stacks:

### Available Templates
- **`react-express`** - React frontend + Express backend (Bun/Node)
- **`react-fastapi`** - React frontend + FastAPI backend (Python)
- **`svelte-fastify`** - SvelteKit + Fastify backend (Bun/Node)
- **`vue-express`** - Vue 3 + Express backend (Bun/Node)
- **`nextjs`** - Next.js 14 full-stack app (Bun/Node)

### Using Templates
```bash
# List available templates
realm templates list

# Create project from template
realm init myapp --template=svelte-fastify

# Provide template variables via CLI
realm init myapp --template=react-express --var project_name=myapp --var author="John Doe"

# Interactive prompts for template variables (if not provided)
realm init --template=vue-express
# Will prompt: > Project name? (myapp)
#             > Author? ()
#             > Description? (A Vue.js and Express application)

# Create your own template
realm create --template=my-stack
```

#### Template Variables

Templates can define custom variables in a `template.yaml` manifest:

```yaml
name: my-template
description: My custom template
variables:
  - name: project_name
    prompt: "Project name"
    default: "{{directory_name}}"
  - name: author
    prompt: "Author"
    default: ""
  - name: api_port
    prompt: "API port"
    default: "3001"
```

Variables are available in template files using Tera syntax:
```json
{
  "name": "{{project_name}}-frontend",
  "author": "{{author}}"
}
```

## Runtime Management

Realm automatically manages runtime versions per project:

```bash
# Use latest Bun (default)
realm init .venv

# Use specific Node.js version
realm init .venv --runtime=node@20

# Use specific Bun version
realm init .venv --runtime=bun@1.0.1

# Use Python with per-project isolation
realm init .venv --runtime=python@3.12
```

### Custom Runtimes

Add any runtime via YAML configuration. Browse [community runtimes](runtimes/):

```bash
# Use Deno
curl -o ~/.realm/runtimes-config/deno.yaml \
  https://raw.githubusercontent.com/wess/realm/main/runtimes/deno.yaml
realm init --runtime=deno

# Use Go
curl -o ~/.realm/runtimes-config/go.yaml \
  https://raw.githubusercontent.com/wess/realm/main/runtimes/go.yaml
realm init --runtime=go
```

See [docs/custom-runtimes.md](docs/custom-runtimes.md) for creating your own.

**Python Support:**
- Downloads and manages Python from python-build-standalone
- Creates per-project `site-packages` for complete isolation
- Automatically sets `VIRTUAL_ENV` for compatibility with pip, poetry, etc.
- Symlinks Python binary from shared installation
- Works seamlessly with existing Python tooling

Runtimes are isolated per realm environment - no global pollution!

## Proxy Server

The built-in proxy intelligently routes requests:

- **Route matching**: `/api/*` → backend:4001, `/` → frontend:4000
- **WebSocket support**: For Vite HMR, live reload, etc.
- **CORS handling**: Automatic CORS headers for development
- **Health checks**: Built-in `/health` endpoint
- **Fallback routing**: Sensible defaults when routes don't match

## Process Management

Realm's process manager handles service lifecycle:

- **Foreman-like**: Define processes in `realm.yml`
- **Intelligent startup**: Processes start in dependency order
- **Log aggregation**: Combined output with process prefixes
- **Graceful shutdown**: Proper process cleanup
- **Auto-restart**: Restart failed processes (optional)

## Deployment

Generate production-ready artifacts:

```bash
realm bundle
```

Creates `dist/` with:
- **Dockerfile** - Multi-stage build for all processes
- **docker-compose.yml** - Complete service orchestration  
- **nginx.conf** - Reverse proxy with your routing
- **deploy.sh** - One-command deployment script

## Architecture

```
┌─────────────────────────────────────────────┐
│                 Realm CLI                   │
├─────────────────────────────────────────────┤
│  Proxy Server (port 8000)                  │
│  ├── Route: /api/* → backend:4001           │
│  ├── Route: / → frontend:4000               │
│  └── Route: /health → built-in              │
├─────────────────────────────────────────────┤
│  Process Manager                            │
│  ├── frontend: bun run dev                  │
│  ├── backend: bun run server                │
│  └── docs: bun run docs                     │
├─────────────────────────────────────────────┤
│  Runtime Manager                            │
│  ├── Bun 1.0.1 (per project)               │
│  └── Node.js 20.5.0 (per project)          │
├─────────────────────────────────────────────┤
│  Environment Manager                        │
│  ├── .env file loading                      │
│  └── Variable isolation                     │
└─────────────────────────────────────────────┘
```

## Commands Reference

### Environment Management
- `realm init [path]` - Create new realm environment
- `source .venv/bin/activate` - Activate environment  
- `deactivate` - Exit realm environment

### Process Management  
- `realm start` - Start all processes + proxy
- `realm stop` - Stop all processes + proxy
- `realm proxy` - Start proxy server only

### Templates
- `realm templates list` - List available templates
- `realm create --template=name` - Create template from current project

### Deployment
- `realm bundle` - Generate deployment artifacts

### Options
- `--runtime=bun|node` - Specify runtime (default: bun)
- `--runtime=node@20` - Specify runtime version
- `--template=name` - Use project template

## Why Realm?

### Before Realm:
```bash
# Terminal 1: Start frontend  
cd frontend && npm run dev

# Terminal 2: Start backend
cd backend && npm run server  

# Terminal 3: Start proxy
nginx -c nginx.conf

# Terminal 4: Set up environment
export NODE_ENV=development
export API_URL=http://localhost:4001
source .env

# Remember all the ports, manage processes, configure nginx...
```

### With Realm:
```bash
realm init .venv --template=react-express
source .venv/bin/activate  
realm start
# Done. Everything runs on http://localhost:8000
```

### The Difference:
- **One command** instead of managing multiple terminals
- **Automatic routing** instead of nginx configuration  
- **Environment isolation** instead of global pollution
- **Template scaffolding** instead of boilerplate setup
- **Deployment generation** instead of Docker wrestling

## Installation

### From Source
```bash
git clone https://github.com/wess/realm
cd realm  
cargo install --path .
```

### Prerequisites
- Rust 1.70+
- Git (for template management)

## Contributing
 
Realm is built in Rust with a modular architecture:

- `src/cli/` - Command-line interface
- `src/config/` - Configuration parsing (`realm.yml`)
- `src/runtime/` - Runtime version management  
- `src/process/` - Process lifecycle management
- `src/proxy/` - HTTP proxy server with routing
- `src/templates/` - Project scaffolding
- `src/bundle/` - Deployment artifact generation
- `tests/` - Comprehensive test suite

### Running Tests
```bash
cargo test
```

### Development
```bash
# Build in development mode
cargo build

# Run with debug output
RUST_LOG=debug cargo run -- start
```

## License

MIT License - see [LICENSE](LICENSE) file.

## Comparison

| Feature | Realm | Docker Compose | Foreman | Create-React-App |
|---------|-------|----------------|---------|------------------|
| Process Management | ✅ | ✅ | ✅ | ❌ |
| Built-in Proxy | ✅ | ❌ | ❌ | ❌ |
| Runtime Isolation | ✅ | ✅ | ❌ | ❌ |
| Project Templates | ✅ | ❌ | ❌ | ✅ |
| Production Deploy | ✅ | ✅ | ❌ | ✅ |
| Zero Config | ✅ | ❌ | ❌ | ✅ |
| Multi-Runtime | ✅ | ✅ | ❌ | ❌ |
| Environment Isolation | ✅ | ✅ | ❌ | ❌ |

Realm combines the best aspects of these tools into a single, cohesive development environment.