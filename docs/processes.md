# Process Management

Realm manages multiple services defined in `realm.yml`, similar to Foreman or Docker Compose.

## Process Definition

Define processes in the `processes` section:

```yaml
processes:
  frontend:
    command: "bun run dev"
    port: 4000
    routes: ["/"]
    working_directory: "frontend"
```

## Required Fields

### command

The command to execute.

```yaml
command: "bun run dev"
```

Can be any shell command:
```yaml
command: "npm run dev"
command: "python -m uvicorn main:app --reload"
command: "docker run --rm -p 5432:5432 postgres"
command: "./start-server.sh"
```

### port

Port the process listens on.

```yaml
port: 4000
```

**Required** unless `routes` is empty (background processes).

The proxy uses this to forward requests to the process.

## Optional Fields

### routes

URL patterns that map to this process.

```yaml
routes: ["/", "/assets/*", "/api/*"]
```

**Default**: `[]`

Patterns:
- `"/"` - Exact or catch-all
- `"/api/*"` - Wildcard match
- `"/health"` - Exact match

Leave empty for background processes (databases, caches):
```yaml
processes:
  database:
    command: "docker run postgres"
    port: 5432
    routes: []  # No HTTP traffic
```

### working_directory

Directory to run the command in.

```yaml
working_directory: "frontend"
```

**Default**: Current directory

Relative to where `realm start` is run.

### env

Process-specific environment variables.

```yaml
env:
  PORT: "4001"
  DATABASE_URL: "postgresql://localhost/myapp"
  LOG_LEVEL: "debug"
```

**Default**: `{}`

Overrides global `env` and `env_file` values.

### dependencies

Processes that must start before this one.

```yaml
dependencies: [database, redis]
```

**Default**: `[]`

Ensures startup order:
```yaml
processes:
  backend:
    command: "bun run server"
    dependencies: [database]

  database:
    command: "docker run postgres"
```

`database` starts first, then `backend`.

## Process Lifecycle

### Startup

When you run `realm start`:

1. **Dependency Resolution**:
   - Builds dependency graph
   - Detects circular dependencies
   - Determines startup order

2. **Process Launch**:
   - Processes start in order
   - Environment variables injected
   - Working directory set
   - Output captured

3. **Monitoring**:
   - Logs combined and prefixed
   - Exit codes monitored
   - Health status tracked

### Running

Realm streams output from all processes:

```
[frontend] Vite dev server running on http://localhost:4000
[backend]  Express server listening on port 4001
[frontend] ✓ ready in 234ms
[database] PostgreSQL init complete
[backend]  Database connected
```

### Shutdown

When you press `Ctrl+C` or run `realm stop`:

1. **Signal Processes**:
   - Sends SIGTERM to all processes
   - Waits for graceful shutdown

2. **Force Kill**:
   - After timeout, sends SIGKILL
   - Ensures all processes exit

3. **Cleanup**:
   - Releases ports
   - Cleans up resources
   - Exits realm

## Common Patterns

### Frontend + Backend

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
    routes: ["/api/*"]
    working_directory: "backend"
```

### With Database

```yaml
processes:
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

### Microservices

```yaml
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
    dependencies: [redis]

  redis:
    command: "docker run --rm -p 6379:6379 redis:alpine"
    port: 6379
    routes: []
```

### Background Workers

```yaml
processes:
  web:
    command: "bun run server"
    port: 4000
    routes: ["/"]

  worker:
    command: "bun run worker"
    port: 0  # No port needed
    routes: []
    env:
      REDIS_URL: "redis://localhost:6379"
```

### Multiple Databases

```yaml
processes:
  backend:
    command: "bun run server"
    port: 4001
    routes: ["/api/*"]
    dependencies: [postgres, redis]

  postgres:
    command: "docker run --rm -p 5432:5432 postgres:15"
    port: 5432
    routes: []

  redis:
    command: "docker run --rm -p 6379:6379 redis:alpine"
    port: 6379
    routes: []
```

## Environment Variables

### Variable Sources

Processes inherit variables from:

1. **System environment**
2. **Global `env` in realm.yml**
3. **`env_file` in realm.yml**
4. **Process-specific `env`**

### Example

```yaml
env:
  NODE_ENV: development
  LOG_LEVEL: info

env_file: .env

processes:
  frontend:
    command: "bun run dev"
    env:
      VITE_API_URL: "http://localhost:8000/api"
    # Gets NODE_ENV=development, LOG_LEVEL=info, VITE_API_URL=...

  backend:
    command: "bun run server"
    env:
      LOG_LEVEL: debug  # Overrides global
      PORT: "4001"
    # Gets NODE_ENV=development, LOG_LEVEL=debug, PORT=4001
```

### Common Variables

**Frontend (Vite)**:
```yaml
env:
  VITE_API_URL: "http://localhost:8000/api"
  VITE_APP_TITLE: "My App"
```

**Backend**:
```yaml
env:
  PORT: "4001"
  DATABASE_URL: "postgresql://localhost/myapp"
  JWT_SECRET: "your-secret"
```

**Python (FastAPI)**:
```yaml
env:
  PYTHON_ENV: "development"
  DATABASE_URL: "postgresql://localhost/myapp"
```

## Dependencies

### Simple Dependencies

```yaml
processes:
  backend:
    command: "bun run server"
    dependencies: [database]

  database:
    command: "docker run postgres"
```

`backend` waits for `database` to start.

### Multiple Dependencies

```yaml
processes:
  backend:
    command: "bun run server"
    dependencies: [database, redis, cache]

  database:
    command: "docker run postgres"

  redis:
    command: "docker run redis"

  cache:
    command: "docker run memcached"
```

`backend` waits for all three to start.

### Chained Dependencies

```yaml
processes:
  web:
    command: "bun run server"
    dependencies: [backend]

  backend:
    command: "bun run api"
    dependencies: [database]

  database:
    command: "docker run postgres"
```

Startup order: `database` → `backend` → `web`

### Validation

Realm detects invalid dependencies:

```yaml
processes:
  backend:
    dependencies: [nonexistent]
# Error: dependency 'nonexistent' not found
```

Circular dependencies:
```yaml
processes:
  frontend:
    dependencies: [backend]

  backend:
    dependencies: [frontend]
# Error: circular dependency detected
```

## Working Directories

### Relative Paths

```yaml
processes:
  frontend:
    command: "bun run dev"
    working_directory: "frontend"

  backend:
    command: "bun run server"
    working_directory: "backend"
```

Paths relative to where `realm start` runs.

### Monorepo Structure

```
project/
├── realm.yml
├── packages/
│   ├── web/
│   ├── mobile/
│   └── shared/
└── services/
    ├── api/
    └── worker/
```

```yaml
processes:
  web:
    command: "bun run dev"
    working_directory: "packages/web"

  mobile:
    command: "bun run dev"
    working_directory: "packages/mobile"

  api:
    command: "bun run server"
    working_directory: "services/api"

  worker:
    command: "bun run start"
    working_directory: "services/worker"
```

## Log Management

### Log Format

Logs are prefixed with process name:

```
[frontend] > vite@4.3.0 dev
[frontend] > vite
[backend]  > nodemon server.js
[frontend] Vite dev server running on http://localhost:4000
[backend]  Express listening on port 4001
```

### Filtering Logs

Since logs are prefixed, you can filter:

```bash
realm start | grep "\[backend\]"
# Only shows backend logs
```

### No Log Prefix

Realm doesn't support disabling prefixes. All output is prefixed for clarity.

## Port Management

### Port Requirements

- Each process with routes must have a unique port
- Ports must be available (not in use)
- Proxy port must not conflict with process ports

```yaml
proxy_port: 8000

processes:
  frontend:
    port: 4000  # Different from proxy

  backend:
    port: 4001  # Different from frontend and proxy
```

### Port Conflicts

```yaml
proxy_port: 8000

processes:
  frontend:
    port: 8000  # Error: conflicts with proxy_port
```

### Dynamic Ports

Some dev servers choose their own port:

```yaml
processes:
  vite:
    command: "vite --port 4000"  # Force specific port
    port: 4000
    routes: ["/"]
```

Always specify the port explicitly.

## Process Failures

### Automatic Restart

Currently, Realm does **not** automatically restart failed processes.

If a process exits:
- Realm logs the exit
- Other processes continue running
- Manual restart required

### Manual Restart

```bash
# Stop everything
realm stop

# Fix the issue

# Start again
realm start
```

### Exit Codes

Processes should exit with appropriate codes:
- `0` - Success
- `1` - General error
- `>1` - Specific error codes

## Advanced Patterns

### Health Checks

Add a health check endpoint:

```yaml
processes:
  backend:
    command: "bun run server"
    port: 4001
    routes: ["/api/*", "/health"]
```

```typescript
// backend: Add health endpoint
app.get('/health', (req, res) => {
  res.json({ status: 'ok' });
});
```

Test with:
```bash
curl http://localhost:8000/health
```

### Multiple Frontends

```yaml
processes:
  web:
    command: "bun run dev"
    port: 4000
    routes: ["/"]
    working_directory: "apps/web"

  admin:
    command: "bun run dev"
    port: 4010
    routes: ["/admin/*"]
    working_directory: "apps/admin"

  backend:
    command: "bun run server"
    port: 4001
    routes: ["/api/*"]
```

- `/` → web app
- `/admin/*` → admin app
- `/api/*` → backend

### Conditional Processes

Use environment variables to conditionally start processes:

```yaml
env_file: .env

processes:
  backend:
    command: "bun run server"
    port: 4001
    routes: ["/api/*"]

  worker:
    command: "[ \"$ENABLE_WORKER\" = \"true\" ] && bun run worker || echo 'Worker disabled'"
    port: 0
    routes: []
```

In `.env`:
```env
ENABLE_WORKER=true
```

### Script Wrappers

For complex startup logic, use scripts:

```yaml
processes:
  backend:
    command: "./scripts/start-backend.sh"
    port: 4001
    routes: ["/api/*"]
```

```bash
#!/bin/bash
# scripts/start-backend.sh

# Wait for database
while ! nc -z localhost 5432; do
  echo "Waiting for database..."
  sleep 1
done

# Run migrations
bun run migrate

# Start server
bun run server
```

## Best Practices

### Use Descriptive Names

```yaml
# Good
processes:
  user_service:
  product_api:
  auth_gateway:

# Avoid
processes:
  app1:
  service2:
```

### Document Dependencies

```yaml
processes:
  backend:
    command: "bun run server"
    dependencies: [database]  # Needs DB connection
```

### Separate Concerns

```yaml
# Good: Separate processes
processes:
  web:
    command: "bun run dev"

  api:
    command: "bun run server"

# Avoid: Everything in one process
processes:
  monolith:
    command: "./start-everything.sh"
```

### Use env_file for Secrets

```yaml
# Don't commit secrets
env_file: .env  # In .gitignore

processes:
  backend:
    command: "bun run server"
    # Gets DATABASE_PASSWORD from .env
```

### Explicit Ports

```yaml
# Good
command: "vite --port 4000"
port: 4000

# Avoid (port mismatch)
command: "vite"  # Picks random port
port: 4000
```

## Next Steps

- [Proxy Server](proxy.md) - Understanding request routing
- [Configuration](configuration.md) - Full realm.yml reference
- [Deployment](deployment.md) - Deploy multiple services
