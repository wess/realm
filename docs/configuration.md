# Configuration

Complete guide to `realm.yml` configuration.

## Overview

Every Realm project uses a `realm.yml` file to define:
- Proxy server settings
- Environment variables
- Process definitions
- Service routing

## Minimal Configuration

The simplest possible configuration:

```yaml
proxy_port: 8000

processes:
  app:
    command: "bun run dev"
    port: 3000
    routes: ["/"]
```

This is enough to run a single-process application.

## Full Configuration Example

```yaml
# Proxy server configuration
proxy_port: 8000

# Global environment variables
env:
  NODE_ENV: development
  LOG_LEVEL: debug
  API_URL: http://localhost:8000/api

# Load additional variables from file
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
      PORT: "4001"
      DATABASE_URL: "postgresql://localhost:5432/myapp"

  database:
    command: "docker run --rm -p 5432:5432 -e POSTGRES_DB=myapp postgres:15"
    port: 5432
    routes: []
```

## Top-Level Fields

### proxy_port

Port for the proxy server to listen on.

```yaml
proxy_port: 8000
```

**Type**: Integer
**Default**: 8000
**Range**: 1024-65535

Access your app at `http://localhost:8000`.

### env

Global environment variables available to all processes.

```yaml
env:
  NODE_ENV: development
  LOG_LEVEL: debug
  API_URL: http://localhost:8000/api
```

**Type**: Map of string → string
**Default**: Empty

Variables are available to all processes unless overridden.

### env_file

Path to a file containing environment variables.

```yaml
env_file: .env
```

**Type**: String (file path)
**Default**: None

File format:
```env
DATABASE_URL=postgresql://localhost/myapp
API_KEY=secret_key_123
LOG_LEVEL=debug
```

Lines starting with `#` are comments. Empty lines are ignored.

### processes

Map of process definitions.

```yaml
processes:
  frontend:
    command: "bun run dev"
    port: 4000
    routes: ["/"]

  backend:
    command: "bun run server"
    port: 4001
    routes: ["/api/*"]
```

**Type**: Map of process name → process config
**Required**: Yes (at least one process)

## Process Configuration

Each process has these fields:

### command

Command to execute.

```yaml
command: "bun run dev"
```

**Type**: String
**Required**: Yes

Can be any shell command:
```yaml
command: "npm run dev"
command: "python -m uvicorn main:app --reload"
command: "docker run --rm -p 5432:5432 postgres"
```

### port

Port the process listens on.

```yaml
port: 4000
```

**Type**: Integer
**Required**: Yes (unless routes is empty)

The proxy uses this to forward requests.

### routes

URL patterns that map to this process.

```yaml
routes: ["/", "/assets/*", "/api/v1/*"]
```

**Type**: Array of strings
**Default**: []

Patterns:
- `"/"` - Exact match (catch-all if no other matches)
- `"/api/*"` - Wildcard (matches /api/anything)
- `"/health"` - Exact match

Route priority (highest to lowest):
1. Exact matches (`/health`)
2. Most specific wildcards (`/api/v1/*`)
3. Less specific wildcards (`/api/*`)
4. Root catch-all (`/`)

### working_directory

Directory to run the command in.

```yaml
working_directory: "frontend"
```

**Type**: String (directory path)
**Default**: Current directory

Relative to where `realm start` is run.

### env

Process-specific environment variables.

```yaml
env:
  PORT: "4001"
  DATABASE_URL: "postgresql://localhost/myapp"
```

**Type**: Map of string → string
**Default**: Empty

Overrides global env and env_file values.

### dependencies

Processes that must start before this one.

```yaml
dependencies: [database, redis]
```

**Type**: Array of process names
**Default**: []

Realm ensures dependencies start first:
```yaml
processes:
  backend:
    command: "bun run server"
    dependencies: [database]

  database:
    command: "docker run postgres"
```

`database` starts before `backend`.

## Environment Variable Precedence

Variables are loaded in this order (later overrides earlier):

1. **System environment** - Already in your shell
2. **Global env** - `env` section in realm.yml
3. **env_file** - File specified by `env_file`
4. **Process env** - Process-specific `env` section

Example:
```yaml
env:
  LOG_LEVEL: info        # Global default

env_file: .env           # Contains LOG_LEVEL=debug

processes:
  frontend:
    command: "bun run dev"
    # Gets LOG_LEVEL=debug from .env

  backend:
    command: "bun run server"
    env:
      LOG_LEVEL: error   # Overrides .env
    # Gets LOG_LEVEL=error
```

## Common Patterns

### Full-Stack App

Frontend and backend with routing:

```yaml
proxy_port: 8000

processes:
  frontend:
    command: "bun run dev"
    port: 4000
    routes: ["/"]
    working_directory: "frontend"

  backend:
    command: "bun run server"
    port: 4001
    routes: ["/api/*", "/health"]
    working_directory: "backend"
```

### Microservices

Multiple backend services:

```yaml
proxy_port: 8000

processes:
  frontend:
    command: "bun run dev"
    port: 4000
    routes: ["/"]

  users_api:
    command: "bun run server"
    port: 4001
    routes: ["/api/users/*"]
    working_directory: "services/users"

  products_api:
    command: "bun run server"
    port: 4002
    routes: ["/api/products/*"]
    working_directory: "services/products"

  auth_api:
    command: "bun run server"
    port: 4003
    routes: ["/api/auth/*"]
    working_directory: "services/auth"
```

### With Database

App with database dependency:

```yaml
proxy_port: 8000

env_file: .env

processes:
  frontend:
    command: "bun run dev"
    port: 4000
    routes: ["/"]

  backend:
    command: "bun run server"
    port: 4001
    routes: ["/api/*"]
    dependencies: [database]
    env:
      DATABASE_URL: "postgresql://localhost:5432/myapp"

  database:
    command: "docker run --rm -p 5432:5432 -e POSTGRES_DB=myapp postgres:15"
    port: 5432
    routes: []
```

### Next.js Full-Stack

Single Next.js application:

```yaml
proxy_port: 8000

env:
  NODE_ENV: development

processes:
  nextjs:
    command: "bun run dev"
    port: 3000
    routes: ["/"]
```

### Python FastAPI

React frontend with FastAPI backend:

```yaml
proxy_port: 8000

env:
  ENVIRONMENT: development

processes:
  frontend:
    command: "bun run dev"
    port: 4000
    routes: ["/"]
    working_directory: "frontend"

  backend:
    command: "python -m uvicorn main:app --reload --port 4001"
    port: 4001
    routes: ["/api/*", "/docs"]
    working_directory: "backend"
```

### With Redis

App with Redis for caching:

```yaml
proxy_port: 8000

processes:
  frontend:
    command: "bun run dev"
    port: 4000
    routes: ["/"]

  backend:
    command: "bun run server"
    port: 4001
    routes: ["/api/*"]
    dependencies: [redis]
    env:
      REDIS_URL: "redis://localhost:6379"

  redis:
    command: "docker run --rm -p 6379:6379 redis:alpine"
    port: 6379
    routes: []
```

## Environment File Format

### .env File

```env
# Database configuration
DATABASE_URL=postgresql://localhost:5432/myapp_dev
DATABASE_POOL_SIZE=10

# API keys
STRIPE_API_KEY=sk_test_123
SENDGRID_API_KEY=SG.456

# Feature flags
ENABLE_ANALYTICS=true
ENABLE_BETA_FEATURES=false

# Logging
LOG_LEVEL=debug
LOG_FORMAT=json

# Empty lines and comments are ignored
```

### Rules

- Format: `KEY=VALUE`
- No spaces around `=`
- No quotes needed for values
- Lines starting with `#` are comments
- Empty lines are ignored
- No variable expansion (use literal values)

### Example Usage

```yaml
env_file: .env

processes:
  backend:
    command: "bun run server"
    # Gets DATABASE_URL, STRIPE_API_KEY, etc. from .env
```

## Route Matching Details

### Exact Match

```yaml
routes: ["/health", "/api/status"]
```

Matches only exact paths:
- ✓ `/health`
- ✓ `/api/status`
- ✗ `/health/check`
- ✗ `/api/status/db`

### Wildcard Match

```yaml
routes: ["/api/*"]
```

Matches path and everything under it:
- ✓ `/api/users`
- ✓ `/api/users/123`
- ✓ `/api/products/search?q=foo`
- ✗ `/api` (no trailing content)

### Root Catch-All

```yaml
routes: ["/"]
```

Matches everything not matched by other routes:
- ✓ `/`
- ✓ `/about`
- ✓ `/products/123`
- Only if no other route matches

### Priority Example

```yaml
processes:
  health:
    routes: ["/health"]           # Priority 1 (exact)

  users_api:
    routes: ["/api/users/*"]      # Priority 2 (specific wildcard)

  general_api:
    routes: ["/api/*"]            # Priority 3 (less specific wildcard)

  frontend:
    routes: ["/"]                 # Priority 4 (catch-all)
```

Request routing:
- `/health` → health (exact match)
- `/api/users/123` → users_api (specific wildcard)
- `/api/products` → general_api (less specific wildcard)
- `/about` → frontend (catch-all)

## Validation

Realm validates your configuration on startup.

### Required Fields

```yaml
proxy_port: 8000  # Required

processes:        # Required (at least one)
  app:
    command: "bun run dev"  # Required
    port: 3000              # Required (unless routes is [])
    routes: ["/"]           # Optional but recommended
```

### Common Errors

**Missing command**:
```yaml
processes:
  frontend:
    port: 4000
    routes: ["/"]
# Error: process 'frontend' missing 'command'
```

**Missing port**:
```yaml
processes:
  backend:
    command: "bun run server"
    routes: ["/api/*"]
# Error: process 'backend' has routes but no port
```

**Invalid dependency**:
```yaml
processes:
  backend:
    command: "bun run server"
    dependencies: [database]
# Error: dependency 'database' not found in processes
```

**Circular dependency**:
```yaml
processes:
  backend:
    command: "bun run server"
    dependencies: [frontend]

  frontend:
    command: "bun run dev"
    dependencies: [backend]
# Error: circular dependency detected
```

## Best Practices

### Use env_file for Secrets

Don't commit secrets to realm.yml:

```yaml
# Good
env_file: .env

# .env (not committed)
DATABASE_PASSWORD=secret123
API_KEY=key_456
```

```yaml
# Bad
env:
  DATABASE_PASSWORD: secret123  # Don't commit this!
```

### Document Environment Variables

Create `.env.example`:

```env
# .env.example (committed)
DATABASE_URL=postgresql://localhost/myapp
API_KEY=your_api_key_here
LOG_LEVEL=info
```

### Use Relative Paths

```yaml
# Good
working_directory: "frontend"
working_directory: "./backend"

# Avoid absolute paths
working_directory: "/Users/you/project/frontend"
```

### Organize by Service

```yaml
processes:
  # Frontend
  frontend:
    command: "bun run dev"
    port: 4000
    routes: ["/"]

  # Backend services
  backend:
    command: "bun run server"
    port: 4001
    routes: ["/api/*"]

  # Infrastructure
  database:
    command: "docker run postgres"
    port: 5432
    routes: []
```

### Keep It Simple

Start minimal and add complexity as needed:

```yaml
# Start here
proxy_port: 8000
processes:
  app:
    command: "bun run dev"
    port: 3000
    routes: ["/"]

# Add complexity later
# - Multiple processes
# - Dependencies
# - Environment files
# - Process-specific env
```

## Next Steps

- [Process Management](processes.md) - Advanced process patterns
- [Proxy Server](proxy.md) - Routing and WebSockets
- [Deployment](deployment.md) - Production configuration
