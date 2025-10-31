# Quick Start

Get up and running with Realm in 5 minutes.

## Interactive Mode (Recommended)

The fastest way to start - Realm will guide you:

```bash
# Run init without flags
realm init

# Realm will prompt you:
# → Project name: myapp
# → Select runtime: Bun (latest)
# → Use a template?: React + Express
# → Done!

cd myapp
source .venv/bin/activate
realm start
```

Visit http://localhost:8000 - your full-stack app is running!

## Onboarding Mode (Existing Projects)

Join an existing project? Use `realm mount` to automatically setup everything:

```bash
# Clone your team's project
git clone https://github.com/yourteam/project
cd project

# Automatically detect and setup environment
realm mount

# Realm automatically:
# → Detects realm.yml, package.json, requirements.txt
# → Infers correct runtime
# → Creates .venv environment
# → Installs all dependencies
# → Copies .env.example to .env

# Activate and start
source .venv/bin/activate
realm start
```

Perfect for:
- New team members onboarding
- Setting up on a new machine
- CI/CD pipelines
- Quick project demos

## Command-Line Mode

For automation or when you know what you want:

```bash
# Create with specific options
realm init myapp --runtime=bun --template=react-express

# Or use defaults (no prompts)
realm init -y

# Activate and start
cd myapp
source .venv/bin/activate
realm start
```

## Your First Project

### Step 1: Initialize

Create a new realm environment:

```bash
realm init myapp --template=react-express
```

This creates:
```
myapp/
├── .venv/              # Realm environment
├── realm.yml           # Configuration
├── project/
│   ├── frontend/       # React app
│   └── backend/        # Express server
```

### Step 2: Activate

Activate the realm environment:

```bash
cd myapp
source .venv/bin/activate
```

Your shell prompt now shows `(realm)`.

### Step 3: Start

Start all services:

```bash
realm start
```

Realm will:
1. Start your frontend dev server
2. Start your backend API server
3. Start the proxy server
4. Route requests intelligently

Output:
```
🚀 Starting realm environment...
🔧 Starting processes...
   → frontend: http://localhost:4000
   → backend: http://localhost:4001
🌐 Starting proxy server...
   → http://localhost:8000
```

### Step 4: Develop

Access your app:
- **Main URL**: http://localhost:8000
- **Frontend**: http://localhost:8000/ → routed to frontend
- **API**: http://localhost:8000/api/* → routed to backend
- **Health**: http://localhost:8000/health

All requests go through the proxy - no CORS issues!

## Common Workflows

### Python Stack

```bash
# Create FastAPI + React project
realm init myapp --template=react-fastapi --runtime=python@3.12

cd myapp
source .venv/bin/activate

# Install Python dependencies
pip install -r backend/requirements.txt

# Start everything
realm start
```

### Specific Runtime Versions

```bash
# Use Node.js 20
realm init myapp --runtime=node@20 --template=vue-express

# Use Bun 1.0.1
realm init myapp --runtime=bun@1.0.1 --template=nextjs

# Use Python 3.11
realm init myapp --runtime=python@3.11
```

### Empty Environment

Start from scratch without a template:

```bash
realm init myapp --runtime=bun

cd myapp
source .venv/bin/activate

# Create your own realm.yml
cat > realm.yml << 'EOF'
proxy_port: 8000

processes:
  app:
    command: "bun run dev"
    port: 3000
    routes: ["/"]
EOF

realm start
```

## Available Templates

- **react-express** - React + Express (Bun/Node)
- **react-fastapi** - React + FastAPI (Python)
- **vue-express** - Vue 3 + Express (Bun/Node)
- **svelte-fastify** - SvelteKit + Fastify (Bun/Node)
- **nextjs** - Next.js 14 full-stack (Bun/Node)

List all templates:
```bash
realm templates list
```

### Template Variables

Templates support custom variables for personalization:

```bash
# Provide variables via CLI
realm init myapp --template=react-express \
  --var project_name=myapp \
  --var author="John Doe"

# Interactive prompts (if not provided)
realm init --template=react-express
# → Project name? myapp
# → Author? John Doe
# → Description? A React and Express application

# Skip prompts, use defaults
realm init --template=react-express --yes
```

Variables are substituted in template files (package.json, README.md, etc.). See [Templates Guide](templates.md#template-variables) for details.

## Managing Runtime Versions

### List Available Versions

```bash
# List Python versions
realm list --runtime=python

# List Node.js versions
realm list --runtime=node

# List Bun versions
realm list --runtime=bun
```

Output shows installed versions with a ✓:
```
📦 Fetching available Python versions...

   Available versions:
   ✓ 3.12.6 (installed)
   • 3.12.5
   • 3.12.4
   • 3.11.9
   ...
```

### Clear Version Cache

Version lists are cached for 24 hours. Clear the cache:

```bash
realm cache clear
```

## Environment Management

### Activate Environment

```bash
source .venv/bin/activate
```

Shell prompt changes to `(realm) $`

### Deactivate Environment

```bash
deactivate
```

### Multiple Environments

Each project gets its own isolated environment:

```bash
# Project 1 with Bun
realm init project1 --runtime=bun
cd project1 && source .venv/bin/activate
realm start
deactivate

# Project 2 with Node.js
realm init project2 --runtime=node@18
cd project2 && source .venv/bin/activate
realm start
```

## Process Control

### Start Everything

```bash
realm start
```

Starts processes + proxy server.

### Start Proxy Only

```bash
realm proxy
```

Useful when managing processes manually.

### Stop Everything

```bash
realm stop
```

Gracefully stops all processes.

## Configuration

After initialization, edit `realm.yml` to customize:

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

## Next Steps

- [Configuration Guide](configuration.md) - Learn about realm.yml
- [Process Management](processes.md) - Define complex service setups
- [Proxy Server](proxy.md) - Understand routing and WebSockets
- [Templates](templates.md) - Create custom templates
- [Deployment](deployment.md) - Deploy to production

## Common Issues

### "Not in an activated realm environment"

```bash
source .venv/bin/activate
```

### Port already in use

Change ports in `realm.yml`:
```yaml
proxy_port: 9000  # Change from 8000

processes:
  frontend:
    port: 5000  # Change from 4000
```

### Process won't start

Check logs for errors. Ensure:
- Dependencies are installed
- Working directory exists
- Port is available

## Getting Help

```bash
# Command help
realm --help
realm init --help

# Version info
realm --version

# List available commands
realm --help
```
