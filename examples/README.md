# Realm Examples

This directory contains example projects demonstrating different use cases for Realm.

## Available Examples

### 1. Basic (`basic/`)

Minimal Realm setup with a single process.

**Structure:**
- Single server running on port 4000
- Proxy on port 8000

**Try it:**
```bash
cd examples/basic
realm init .venv
source .venv/bin/activate
realm start
# Visit http://localhost:8000
```

### 2. Fullstack (`fullstack/`)

Frontend + Backend with intelligent routing.

**Structure:**
- Frontend (Bun server) on port 4000
- Backend (API server) on port 4001
- Proxy routes `/api/*` → backend, everything else → frontend

**Try it:**
```bash
cd examples/fullstack
realm init .venv
source .venv/bin/activate
realm start
# Visit http://localhost:8000
# Click the button to fetch from /api/hello
```

### 3. Monorepo (`monorepo/`)

Multiple services with complex routing.

**Structure:**
- Frontend on port 4000
- API on port 4001
- Docs on port 4002
- Admin on port 4003
- Proxy intelligently routes to each service

**Try it:**
```bash
cd examples/monorepo
realm init .venv
source .venv/bin/activate
realm start
# Visit http://localhost:8000 - frontend
# Visit http://localhost:8000/api - API
# Visit http://localhost:8000/docs - docs
# Visit http://localhost:8000/admin - admin panel
```

## Requirements

- Realm installed (`cargo install realmenv`)
- Bun runtime (installed automatically by Realm)

## Common Commands

```bash
# Initialize environment
realm init .venv

# Activate environment
source .venv/bin/activate

# Start all services
realm start

# Stop all services
realm stop

# Start only proxy
realm proxy

# Generate Docker deployment
realm bundle
```

## Customization

Each example contains a `realm.yml` file. Edit this to:
- Change ports
- Modify routes
- Add environment variables
- Configure working directories
- Add more processes

See the [main README](../README.md) for full configuration options.
