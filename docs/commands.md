# Commands Reference

Complete reference for all Realm commands.

## realm init

Initialize a new realm environment.

### Usage

```bash
realm init [PATH] [OPTIONS]
```

### Arguments

- `PATH` - Directory for the realm environment (default: `.venv`)

### Options

- `--runtime=<SPEC>` - Runtime to use (bun, node, python, bun@1.0.1, node@20, python@3.12)
- `--template=<NAME>` - Template to use for project scaffolding
- `--var <KEY>=<VALUE>` - Set template variable (can be used multiple times)
- `-y, --yes` - Skip interactive prompts and use defaults

### Examples

**Interactive mode (prompts for choices)**:
```bash
realm init
```

**With all options**:
```bash
realm init myapp --runtime=bun@1.1.34 --template=react-express
```

**With template variables**:
```bash
realm init myapp --template=react-express \
  --var project_name=myapp \
  --var author="John Doe" \
  --var description="My awesome app"
```

**Interactive prompts for template variables**:
```bash
realm init --template=react-express
# Will prompt for: project_name, author, description
```

**Quick start with defaults**:
```bash
realm init -y
```

**Specific runtime version**:
```bash
realm init --runtime=node@20
```

**Python project**:
```bash
realm init myapp --runtime=python@3.12 --template=react-fastapi
```

**Empty environment**:
```bash
realm init myapp --runtime=bun
```

### Runtime Specification

Format: `runtime[@version]`

- `bun` - Latest Bun
- `node` - Latest Node.js
- `python` - Latest Python
- `bun@1.0.1` - Specific Bun version
- `node@20` - Latest Node.js 20.x
- `python@3.12` - Latest Python 3.12.x
- `python@3.12.6` - Exact Python version

### What It Creates

```
PATH/
├── .venv/              # Realm environment
│   ├── bin/
│   │   ├── activate   # Activation script
│   │   └── realm      # Runtime symlink
│   ├── runtimes/      # Runtime installations
│   └── lib/           # Realm libraries
├── realm.yml          # Configuration (if using template)
└── project/           # Project files (if using template)
```

### Interactive Mode

When run without `--runtime`, Realm prompts you:

```
🏗️  Create a new Realm environment

Project name: myapp
Select runtime:
  > Bun (latest)
    Node.js (latest)
    Python (latest)
    Bun (specific version)
    Node.js (specific version)
    Python (specific version)

Use a project template?:
  > No template
    React + Express
    React + FastAPI
    Vue + Express
    Svelte + Fastify
    Next.js
```

### Exit Status

- `0` - Success
- `1` - Error (invalid runtime, template not found, etc.)

---

## realm start

Start all processes and proxy server.

### Usage

```bash
realm start
```

### Prerequisites

- Must be in an activated realm environment
- `realm.yml` must exist in current directory

### What It Does

1. Loads `realm.yml` configuration
2. Loads environment variables (from `env` and `env_file`)
3. Starts all defined processes in dependency order
4. Starts proxy server on configured port

### Output

```
🚀 Starting realm environment...
🔧 Starting processes...
   → frontend: http://localhost:4000
   → backend: http://localhost:4001
🌐 Starting proxy server...
   → http://localhost:8000
```

Process logs are prefixed and combined:
```
[frontend] Vite dev server running
[backend]  Express listening on 4001
```

### Exit

Press `Ctrl+C` to gracefully stop all processes and the proxy.

### Exit Status

- `0` - Clean shutdown
- `1` - Error (not activated, config not found, port in use, etc.)

---

## realm stop

Stop all processes and proxy server.

### Usage

```bash
realm stop
```

### Prerequisites

- Must be in an activated realm environment
- `realm.yml` must exist

### What It Does

1. Sends termination signal to all running processes
2. Waits for graceful shutdown
3. Cleans up resources

### Exit Status

- `0` - Success
- `1` - Error (not activated, no processes running, etc.)

---

## realm proxy

Start proxy server only.

### Usage

```bash
realm proxy
```

### Prerequisites

- Must be in an activated realm environment
- `realm.yml` must exist

### What It Does

Starts only the HTTP proxy server without starting processes. Useful when you want to manage processes manually.

### Example

```bash
# Terminal 1: Start processes manually
cd frontend && bun run dev

# Terminal 2: Start backend manually
cd backend && bun run server

# Terminal 3: Start proxy
realm proxy
```

### Exit Status

- `0` - Clean shutdown
- `1` - Error (not activated, port in use, etc.)

---

## realm bundle

Generate deployment bundle with Docker artifacts.

### Usage

```bash
realm bundle
```

### Prerequisites

- Must be in an activated realm environment
- `realm.yml` must exist

### What It Creates

```
dist/
├── Dockerfile           # Multi-stage build for all services
├── docker-compose.yml   # Service orchestration
├── nginx.conf          # Reverse proxy with your routes
└── deploy.sh           # One-command deployment script
```

### Output

```
📦 Creating deployment bundle...
   ✓ Generated Dockerfile
   ✓ Generated docker-compose.yml
   ✓ Generated nginx.conf
   ✓ Generated deploy.sh

Deployment bundle created in: dist/

To deploy:
  cd dist
  ./deploy.sh
```

### Deploy

```bash
cd dist
./deploy.sh
```

This builds and starts your services with Docker Compose.

### Exit Status

- `0` - Success
- `1` - Error (not activated, config invalid, etc.)

---

## realm create

Create a template from the current project.

### Usage

```bash
realm create --template=<NAME>
```

### Options

- `--template=<NAME>` - Name for the template (required)

### Examples

```bash
# Create template from current directory
realm create --template=my-stack

# Use it later
realm init newproject --template=my-stack
```

### What It Does

1. Copies current project structure to `~/.realm/templates/<NAME>/`
2. Includes all files except `.venv/`, `node_modules/`, etc.
3. Template becomes available for `realm init`

### Exit Status

- `0` - Success
- `1` - Error (template exists, invalid name, etc.)

---

## realm templates list

List all available templates.

### Usage

```bash
realm templates list
```

### Output

```
📄 Available templates:
   • react-express (built-in)
   • react-fastapi (built-in)
   • vue-express (built-in)
   • svelte-fastify (built-in)
   • nextjs (built-in)
   • my-stack (custom)
```

### Exit Status

- `0` - Success

---

## realm list

List available runtime versions.

### Usage

```bash
realm list --runtime=<RUNTIME>
```

### Options

- `--runtime=<RUNTIME>` - Runtime to list (bun, node, python) (required)

### Examples

```bash
# List Python versions
realm list --runtime=python

# List Node.js versions
realm list --runtime=node

# List Bun versions
realm list --runtime=bun
```

### Output

```
📦 Fetching available Python versions...
   (using cached data)

   Available versions:
   ✓ 3.12.6 (installed)
   • 3.12.5
   • 3.12.4
   • 3.11.9
   • 3.11.8
   ...
```

Installed versions are marked with ✓.

### Caching

Version lists are cached for 24 hours. Use `realm cache clear` to force a refresh.

### Exit Status

- `0` - Success
- `1` - Error (invalid runtime, network error, etc.)

---

## realm cache clear

Clear cached runtime version lists.

### Usage

```bash
realm cache clear
```

### What It Does

Removes cached version lists from `~/.realm/cache/`, forcing fresh fetches on next `realm list` command.

### Output

```
🗑️ Clearing runtime version cache...
✓ Cache cleared successfully
```

### Exit Status

- `0` - Success
- `1` - Error (cache directory error, etc.)

---

## realm completions

Generate shell completion scripts.

### Usage

```bash
realm completions <SHELL>
```

### Arguments

- `SHELL` - Shell type (bash, zsh, fish, powershell, elvish)

### Examples

**Bash**:
```bash
realm completions bash | sudo tee /etc/bash_completion.d/realm
```

**Zsh**:
```bash
realm completions zsh > ~/.zfunc/_realm
```

**Fish**:
```bash
realm completions fish > ~/.config/fish/completions/realm.fish
```

### What It Does

Generates completion script that enables tab-completion for:
- Commands (init, start, stop, etc.)
- Options (--runtime, --template, etc.)
- Runtime names (bun, node, python)
- Template names (react-express, vue-express, etc.)

### Exit Status

- `0` - Success

---

## Environment Commands

These aren't Realm commands, but part of the workflow.

### source .venv/bin/activate

Activate realm environment.

**Usage**:
```bash
source .venv/bin/activate
```

**Effect**:
- Prepends `.venv/bin` to PATH
- Sets `REALM_ENV` variable
- Modifies prompt to show `(realm)`

**Shell-specific**:
- Bash/Zsh: `source .venv/bin/activate`
- Fish: `source .venv/bin/activate.fish`
- Windows CMD: `.venv\Scripts\activate.bat`
- PowerShell: `.venv\Scripts\Activate.ps1`

### deactivate

Exit realm environment.

**Usage**:
```bash
deactivate
```

**Effect**:
- Restores original PATH
- Removes `REALM_ENV` variable
- Restores original prompt

---

## Global Options

Available for all commands:

- `-h, --help` - Show help message
- `-V, --version` - Show version number

### Examples

```bash
# Show help
realm --help
realm init --help

# Show version
realm --version
```

---

## Exit Status Codes

All Realm commands follow these conventions:

- `0` - Success
- `1` - General error (invalid arguments, config error, etc.)
- `2` - Network error (download failed, timeout, etc.)
- `130` - Interrupted by user (Ctrl+C)

---

## Environment Variables

Realm respects these environment variables:

- `REALM_ENV` - Set when environment is activated
- `VIRTUAL_ENV` - Set for Python compatibility
- `PATH` - Modified to include `.venv/bin`

---

## Next Steps

- [Configuration Guide](configuration.md) - Learn about realm.yml
- [Process Management](processes.md) - Define complex services
- [Templates](templates.md) - Create and use templates
- [Deployment](deployment.md) - Deploy with Docker
