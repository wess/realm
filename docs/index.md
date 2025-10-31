# Realm Documentation

Realm is a zero-config development environment that eliminates the complexity of modern full-stack development.

## What is Realm?

Realm combines isolated runtime environments, intelligent process management, and automatic HTTP routing into a single tool. Think of it as virtualenv + Foreman + nginx, but simpler and runtime-agnostic.

### Key Features

- **Isolated Environments** - Per-project runtime isolation (Bun, Node.js, Python)
- **Process Management** - Define and run multiple services with dependencies
- **Intelligent Proxy** - Automatic request routing with WebSocket support
- **Zero Configuration** - Works out of the box with sensible defaults
- **Template System** - Bootstrap projects with pre-configured stacks and custom variables
- **One-Command Deploy** - Generate production Docker artifacts

### Why Realm?

**Before Realm:**
```bash
# Terminal 1
cd frontend && npm run dev

# Terminal 2
cd backend && npm run server

# Terminal 3
nginx -c nginx.conf

# Terminal 4
export NODE_ENV=development && source .env
```

**With Realm:**
```bash
realm init myapp --template=react-express
cd myapp && source .venv/bin/activate
realm start
# Everything running on http://localhost:8000
```

## Documentation Structure

### Getting Started
- [Installation](installation.md) - Install Realm on your system
- [Quick Start](quickstart.md) - Get up and running in 5 minutes
- [Core Concepts](concepts.md) - Understand how Realm works

### Features
- [Commands Reference](commands.md) - All available commands
- [Configuration](configuration.md) - Configure realm.yml
- [Templates](templates.md) - Use and create project templates
- [Runtime Management](runtimes.md) - Manage Bun, Node.js, and Python versions
- [Process Management](processes.md) - Define and manage services
- [Proxy Server](proxy.md) - HTTP routing and WebSocket support

### Advanced
- [Deployment](deployment.md) - Deploy to production with Docker
- [Advanced Usage](advanced.md) - Custom workflows and patterns
- [Troubleshooting](troubleshooting.md) - Common issues and solutions

## Quick Links

- [GitHub Repository](https://github.com/wess/realm)
- [Issue Tracker](https://github.com/wess/realm/issues)
- [Latest Release](https://github.com/wess/realm/releases/latest)

## Community

Found a bug? Have a feature request? [Open an issue](https://github.com/wess/realm/issues/new).
