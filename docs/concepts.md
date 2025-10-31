# Core Concepts

Understanding how Realm works will help you get the most out of it.

## Isolated Environments

Like Python's virtualenv, Realm creates isolated development environments for each project.

### What Gets Isolated?

- **Runtimes**: Each project can use different runtime versions
- **Dependencies**: npm/bun packages and Python site-packages are project-local
- **Environment Variables**: Set per-project without affecting global system
- **Configuration**: realm.yml defines project-specific settings

### How It Works

```bash
realm init myapp
```

Creates:
```
myapp/
└── .venv/
    ├── bin/
    │   ├── activate       # Activation script
    │   ├── bun           # Symlink to runtime
    │   └── node          # (if using Node)
    ├── runtimes/         # Runtime installations
    └── lib/              # Realm libraries
```

When activated, `PATH` is modified to use the project's runtime:
```bash
source .venv/bin/activate
which bun  # → /path/to/myapp/.venv/bin/bun
```

### Benefits

- **No version conflicts** between projects
- **Reproducible environments** - same runtime version everywhere
- **Clean system** - no global installations polluting PATH
- **Easy cleanup** - delete .venv to remove everything

## Runtime Management

Realm automatically manages Bun, Node.js, and Python installations.

### Runtime Sources

- **Bun**: Downloaded from [bun.sh](https://bun.sh)
- **Node.js**: Downloaded from [nodejs.org](https://nodejs.org)
- **Python**: Downloaded from [python-build-standalone](https://github.com/indygreg/python-build-standalone)

### Version Resolution

```bash
# Latest version
realm init --runtime=bun
# → Resolves to latest stable (e.g., 1.1.34)

# Specific version
realm init --runtime=node@20
# → Installs Node.js 20.x.x (latest 20.x)

# Exact version
realm init --runtime=python@3.12.6
# → Installs exactly Python 3.12.6
```

### System vs Downloaded Runtimes

Realm prefers system-installed runtimes when available:

```bash
realm init --runtime=bun
# If bun is in PATH → uses system bun
# Otherwise → downloads and installs bun to ~/.realm
```

Specific versions always download:
```bash
realm init --runtime=node@20
# Always downloads Node.js 20.x to ~/.realm
```

### Storage Location

Downloaded runtimes are stored in:
```
~/.realm/
├── runtimes/
│   ├── bun-1.1.34/
│   ├── node-20.18.0/
│   └── python-3.12.6/
└── cache/
    └── versions.json  # Cached version lists
```

Multiple projects can share the same runtime installation.

## Process Management

Realm runs and manages multiple services defined in `realm.yml`.

### Process Definition

```yaml
processes:
  frontend:
    command: "bun run dev"
    port: 4000
    routes: ["/"]
    working_directory: "frontend"
```

Each process defines:
- **command**: What to run
- **port**: Which port the service listens on
- **routes**: URL patterns that map to this service
- **working_directory**: Where to run the command

### Process Lifecycle

1. **Startup**: Processes start in dependency order
2. **Running**: Realm monitors process output
3. **Shutdown**: Graceful termination on stop

### Dependencies

Ensure processes start in order:

```yaml
processes:
  backend:
    command: "bun run server"
    dependencies: [database]

  database:
    command: "docker run postgres"
```

`backend` won't start until `database` is running.

### Process Environment

Each process inherits:
1. System environment
2. Global `env` from realm.yml
3. Variables from `env_file`
4. Process-specific `env`

```yaml
env:
  NODE_ENV: development

env_file: .env

processes:
  backend:
    command: "bun run server"
    env:
      PORT: "4001"
      DATABASE_URL: "postgresql://localhost/myapp"
```

### Log Aggregation

Realm combines output from all processes with prefixes:

```
[frontend] Vite dev server running on http://localhost:4000
[backend]  Express server listening on port 4001
[frontend] ✓ ready in 234ms
[backend]  Database connected
```

## Proxy Server

The built-in HTTP proxy routes requests to your services.

### Why a Proxy?

Without a proxy:
```
Frontend: http://localhost:4000
Backend:  http://localhost:4001
→ CORS issues
→ Different ports
→ No unified endpoint
```

With Realm's proxy:
```
Everything: http://localhost:8000
→ No CORS (same origin)
→ Single port
→ Production-like setup
```

### Route Matching

Routes are matched from most to least specific:

```yaml
processes:
  frontend:
    routes: ["/"]          # Catch-all (lowest priority)

  backend:
    routes: ["/api/*"]     # Wildcard (medium priority)

  health:
    routes: ["/health"]    # Exact match (highest priority)
```

Request flow:
- `/health` → health service (exact match)
- `/api/users` → backend (wildcard match)
- `/about` → frontend (catch-all)

### WebSocket Support

The proxy automatically upgrades WebSocket connections:

```yaml
processes:
  frontend:
    port: 4000
    routes: ["/"]
```

Vite's HMR WebSocket at `ws://localhost:8000` automatically proxies to the frontend dev server.

### CORS Handling

The proxy adds CORS headers automatically for development:
```
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS
Access-Control-Allow-Headers: Content-Type, Authorization
```

## Configuration

Everything is configured via `realm.yml`.

### Minimal Configuration

```yaml
proxy_port: 8000

processes:
  app:
    command: "bun run dev"
    port: 3000
    routes: ["/"]
```

### Full Configuration

```yaml
# Proxy configuration
proxy_port: 8000

# Global environment variables
env:
  NODE_ENV: development
  LOG_LEVEL: debug

# Load additional env from file
env_file: .env

# Process definitions
processes:
  frontend:
    command: "bun run dev"
    port: 4000
    routes: ["/", "/assets/*"]
    working_directory: "frontend"
    env:
      VITE_API_URL: "http://localhost:8000/api"

  backend:
    command: "bun run server"
    port: 4001
    routes: ["/api/*", "/health"]
    working_directory: "backend"
    dependencies: [database]
    env:
      DATABASE_URL: "postgresql://localhost/myapp"

  database:
    command: "docker run --rm -p 5432:5432 postgres:15"
    port: 5432
    routes: []
```

### Configuration Precedence

Environment variables are loaded in order:

1. **System environment** - Already in your shell
2. **realm.yml env** - Global project vars
3. **env_file** - Variables from .env
4. **Process env** - Process-specific vars

Later values override earlier ones.

## Templates

Templates are pre-configured project structures.

### Built-in Templates

Realm includes templates for popular stacks:
- **react-express** - React + Express (Bun/Node)
- **react-fastapi** - React + FastAPI (Python)
- **vue-express** - Vue 3 + Express (Bun/Node)
- **svelte-fastify** - SvelteKit + Fastify (Bun/Node)
- **nextjs** - Next.js 14 full-stack (Bun/Node)

### Template Structure

A template contains:
```
template-name/
├── realm.yml          # Process configuration
├── frontend/          # Frontend code
├── backend/           # Backend code
└── .gitignore
```

When you init from a template:
```bash
realm init myapp --template=react-express
```

Realm:
1. Creates the realm environment (.venv)
2. Copies template files to project/
3. Sets up the runtime
4. Ready to start

### Custom Templates

Create templates from your own projects:

```bash
# In your project directory
realm create --template=my-stack

# Use it later
realm init newproject --template=my-stack
```

Templates are stored in `~/.realm/templates/`.

## Deployment

Realm generates production deployment artifacts.

### Bundle Command

```bash
realm bundle
```

Creates:
```
dist/
├── Dockerfile           # Multi-stage build
├── docker-compose.yml   # Service orchestration
├── nginx.conf          # Reverse proxy config
└── deploy.sh           # Deployment script
```

### How It Works

Realm reads your `realm.yml` and generates:

1. **Dockerfile** - Builds all services in one image
2. **docker-compose.yml** - Defines service networking
3. **nginx.conf** - Replicates your route configuration
4. **deploy.sh** - Single command to deploy

### Production Architecture

Development:
```
realm start
→ Realm proxy routes requests
```

Production:
```
docker-compose up
→ nginx routes requests
→ Services run in containers
```

Same routing logic, production-ready infrastructure.

## Environment Activation

Realm uses shell scripts to modify your environment.

### What Activation Does

```bash
source .venv/bin/activate
```

1. Prepends `.venv/bin` to PATH
2. Sets `REALM_ENV` variable
3. Sets `VIRTUAL_ENV` variable (Python compatibility)
4. Modifies shell prompt to show `(realm)`

### Activation Script

The activate script is shell-specific:
- `.venv/bin/activate` - Bash/Zsh
- `.venv/bin/activate.fish` - Fish
- `.venv/Scripts/activate.bat` - Windows CMD
- `.venv/Scripts/Activate.ps1` - PowerShell

### Deactivation

```bash
deactivate
```

Restores:
- Original PATH
- Original prompt
- Removes REALM_ENV variable

### Multiple Environments

You can only activate one environment at a time:

```bash
source project1/.venv/bin/activate
# Now in project1 environment

source project2/.venv/bin/activate
# Switches to project2 environment
```

## Caching

Realm caches data to improve performance.

### Version Lists

When you run:
```bash
realm list --runtime=python
```

Realm fetches available versions and caches them for 24 hours.

Cache location: `~/.realm/cache/`

### Cache Management

Clear cached data:
```bash
realm cache clear
```

This forces fresh fetches on next command.

## Next Steps

- [Commands Reference](commands.md) - Learn all commands
- [Configuration Guide](configuration.md) - Deep dive into realm.yml
- [Process Management](processes.md) - Advanced process patterns
- [Proxy Server](proxy.md) - Routing and WebSockets
