# Advanced Usage

Advanced patterns and workflows for power users.

## Complex Project Structures

### Monorepo Setup

Manage multiple apps in one repository:

```
monorepo/
├── realm.yml
├── apps/
│   ├── web/
│   ├── mobile/
│   └── admin/
├── services/
│   ├── api/
│   ├── auth/
│   └── worker/
└── packages/
    ├── shared/
    └── ui/
```

```yaml
# realm.yml
proxy_port: 8000

processes:
  web:
    command: "bun run dev"
    port: 4000
    routes: ["/"]
    working_directory: "apps/web"

  mobile:
    command: "bun run dev"
    port: 4010
    routes: ["/mobile/*"]
    working_directory: "apps/mobile"

  admin:
    command: "bun run dev"
    port: 4020
    routes: ["/admin/*"]
    working_directory: "apps/admin"

  api:
    command: "bun run dev"
    port: 4001
    routes: ["/api/*"]
    working_directory: "services/api"
    dependencies: [database]

  auth:
    command: "bun run dev"
    port: 4002
    routes: ["/auth/*"]
    working_directory: "services/auth"

  worker:
    command: "bun run start"
    port: 0
    routes: []
    working_directory: "services/worker"
    dependencies: [redis]

  database:
    command: "docker run --rm -p 5432:5432 -e POSTGRES_DB=monorepo postgres:15"
    port: 5432
    routes: []

  redis:
    command: "docker run --rm -p 6379:6379 redis:alpine"
    port: 6379
    routes: []
```

### Multi-Environment Configs

Different configs for dev/staging/prod:

```
project/
├── realm.dev.yml
├── realm.staging.yml
├── realm.prod.yml
└── .env.dev
```

Use with environment variable:

```bash
# Development
export REALM_ENV=dev
realm start  # Uses realm.dev.yml

# Staging
export REALM_ENV=staging
realm start  # Uses realm.staging.yml
```

Or symlink:
```bash
ln -s realm.dev.yml realm.yml
realm start
```

### Nested Workspaces

Run Realm inside Realm (not recommended, but possible):

```yaml
# Main realm.yml
processes:
  frontend:
    command: "cd frontend && source .venv/bin/activate && realm start"
    port: 4000
    routes: ["/"]

  backend:
    command: "cd backend && source .venv/bin/activate && realm start"
    port: 4001
    routes: ["/api/*"]
```

## Advanced Process Patterns

### Conditional Processes

Use shell scripting for conditional startup:

```yaml
processes:
  backend:
    command: "[ \"$RUN_MIGRATIONS\" = \"true\" ] && bun run migrate && bun run server || bun run server"
    port: 4001
    routes: ["/api/*"]
```

```bash
export RUN_MIGRATIONS=true
realm start
```

### Process Pooling

Run multiple instances of the same service:

```yaml
processes:
  worker1:
    command: "bun run worker"
    port: 0
    routes: []
    env:
      WORKER_ID: "1"

  worker2:
    command: "bun run worker"
    port: 0
    routes: []
    env:
      WORKER_ID: "2"

  worker3:
    command: "bun run worker"
    port: 0
    routes: []
    env:
      WORKER_ID: "3"
```

### Startup Scripts

Complex initialization logic:

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
set -e

echo "Waiting for dependencies..."
while ! nc -z localhost 5432; do sleep 1; done
while ! nc -z localhost 6379; do sleep 1; done

echo "Running migrations..."
bun run migrate

echo "Seeding database..."
bun run seed

echo "Starting server..."
exec bun run server
```

Make executable:
```bash
chmod +x scripts/start-backend.sh
```

### Health Check Polling

Wait for service health before starting dependents:

```bash
#!/bin/bash
# scripts/wait-for-backend.sh

until curl -f http://localhost:4001/health; do
  echo "Waiting for backend..."
  sleep 2
done

echo "Backend is ready!"
exec bun run server
```

```yaml
processes:
  backend:
    command: "bun run server"
    port: 4001
    routes: ["/api/*"]

  worker:
    command: "./scripts/wait-for-backend.sh"
    port: 0
    routes: []
```

## Advanced Routing

### API Versioning

Route different API versions to different services:

```yaml
processes:
  frontend:
    port: 4000
    routes: ["/"]

  api_v1:
    command: "bun run server-v1"
    port: 4001
    routes: ["/api/v1/*"]
    working_directory: "backend/v1"

  api_v2:
    command: "bun run server-v2"
    port: 4002
    routes: ["/api/v2/*"]
    working_directory: "backend/v2"

  api_v3:
    command: "bun run server-v3"
    port: 4003
    routes: ["/api/v3/*"]
    working_directory: "backend/v3"
```

### Feature Flags with Routing

Route to different implementations:

```yaml
processes:
  frontend_new:
    command: "bun run dev"
    port: 4000
    routes: ["/beta/*"]
    working_directory: "frontend-next"

  frontend_old:
    command: "bun run dev"
    port: 4010
    routes: ["/"]
    working_directory: "frontend"
```

### GraphQL + REST

Multiple API styles:

```yaml
processes:
  frontend:
    port: 4000
    routes: ["/"]

  rest_api:
    command: "bun run rest-server"
    port: 4001
    routes: ["/api/*"]

  graphql:
    command: "bun run graphql-server"
    port: 4002
    routes: ["/graphql"]
```

## Environment Management

### Encrypted Secrets

Use tools like `sops` or `age`:

```bash
# Encrypt .env
age -e -o .env.age -i ~/.age/key.txt .env

# Decrypt and load
age -d -i ~/.age/key.txt .env.age > .env
realm start
```

### Dynamic Environment Loading

Load different env files:

```yaml
env_file: ".env.$USER"  # Per-user configs
```

```bash
# Create user-specific env
echo "LOG_LEVEL=debug" > .env.alice
echo "LOG_LEVEL=info" > .env.bob

# Use based on username
realm start
```

### Secret Injection

Use external secret managers:

```bash
#!/bin/bash
# scripts/start-with-secrets.sh

# Fetch from AWS Secrets Manager
export DATABASE_PASSWORD=$(aws secretsmanager get-secret-value \
  --secret-id prod/db/password \
  --query SecretString \
  --output text)

# Start realm
exec realm start
```

## Template Development

### Template with Variables

Create templates with placeholders:

```yaml
# template/realm.yml
proxy_port: 8000

env:
  APP_NAME: "{{APP_NAME}}"
  ENVIRONMENT: "{{ENVIRONMENT}}"

processes:
  frontend:
    command: "bun run dev"
    port: 4000
    routes: ["/"]
```

Custom init script:

```bash
#!/bin/bash
# scripts/custom-init.sh

APP_NAME=$1
ENVIRONMENT=${2:-development}

realm init --template=my-template
cd .venv

sed -i "s/{{APP_NAME}}/$APP_NAME/g" ../realm.yml
sed -i "s/{{ENVIRONMENT}}/$ENVIRONMENT/g" ../realm.yml

echo "Initialized $APP_NAME for $ENVIRONMENT"
```

### Template Testing

Test templates before sharing:

```bash
#!/bin/bash
# scripts/test-template.sh

TEMPLATE=$1
TMP_DIR=$(mktemp -d)

cd "$TMP_DIR"
realm init test --template="$TEMPLATE"
cd test
source .venv/bin/activate

# Install dependencies
cd project/frontend && bun install
cd ../backend && bun install
cd ../..

# Start and test
timeout 30s realm start &
PID=$!

sleep 10

# Test endpoints
curl -f http://localhost:8000/ || exit 1
curl -f http://localhost:8000/api/health || exit 1

kill $PID
echo "✓ Template $TEMPLATE works!"
```

## Development Workflows

### Hot Reload Everything

Watch and reload on changes:

```yaml
processes:
  frontend:
    command: "bun run dev --reload"
    port: 4000
    routes: ["/"]

  backend:
    command: "bun run --watch src/server.ts"
    port: 4001
    routes: ["/api/*"]

  docs:
    command: "bun run docs:watch"
    port: 4002
    routes: ["/docs/*"]
```

### Testing in Realm

Run tests alongside services:

```yaml
processes:
  frontend:
    command: "bun run dev"
    port: 4000
    routes: ["/"]

  backend:
    command: "bun run dev"
    port: 4001
    routes: ["/api/*"]

  test:
    command: "bun run test:watch"
    port: 0
    routes: []
    dependencies: [frontend, backend]
```

### Debugging

Debug with inspector:

```yaml
processes:
  backend:
    command: "bun run --inspect server.ts"
    port: 4001
    routes: ["/api/*"]
    env:
      NODE_OPTIONS: "--inspect=9229"
```

Connect debugger to `localhost:9229`.

### Development Scripts

Combine multiple operations:

```bash
#!/bin/bash
# scripts/dev.sh

# Install dependencies
echo "Installing dependencies..."
(cd frontend && bun install)
(cd backend && bun install)

# Run migrations
echo "Running migrations..."
bun run migrate

# Start realm
echo "Starting development environment..."
realm start
```

```bash
chmod +x scripts/dev.sh
./scripts/dev.sh
```

## Performance Optimization

### Parallel Startup

Use `&` for true parallelism:

```bash
#!/bin/bash
# scripts/parallel-start.sh

(cd frontend && bun run dev) &
(cd backend && bun run server) &
realm proxy  # Only run proxy

wait
```

### Lazy Loading

Start expensive services on-demand:

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

  # database:  # Comment out when not needed
  #   command: "docker run postgres"
  #   port: 5432
```

### Resource Limits

Limit process resources:

```yaml
processes:
  backend:
    command: "docker run --memory=512m --cpus=0.5 mybackend"
    port: 4001
    routes: ["/api/*"]
```

## Integration with Other Tools

### With Docker Compose

Use realm for development, compose for deps:

```yaml
# docker-compose.yml
services:
  postgres:
    image: postgres:15
    ports:
      - "5432:5432"
    environment:
      POSTGRES_DB: myapp

  redis:
    image: redis:alpine
    ports:
      - "6379:6379"
```

```yaml
# realm.yml
processes:
  frontend:
    command: "bun run dev"
    port: 4000
    routes: ["/"]

  backend:
    command: "bun run server"
    port: 4001
    routes: ["/api/*"]
    env:
      DATABASE_URL: "postgresql://localhost:5432/myapp"
      REDIS_URL: "redis://localhost:6379"
```

```bash
# Start dependencies
docker-compose up -d

# Start app
source .venv/bin/activate
realm start
```

### With Kubernetes

Use realm locally, K8s in production:

```bash
# Development
realm start

# Production
realm bundle
# Convert to K8s
kompose convert -f dist/docker-compose.yml
kubectl apply -f .
```

### With Terraform

Infrastructure as code:

```hcl
# terraform/main.tf
resource "aws_ecs_service" "app" {
  name            = "myapp"
  cluster         = aws_ecs_cluster.main.id
  task_definition = aws_ecs_task_definition.app.arn
  desired_count   = 3
}
```

```bash
# Build and push image
realm bundle
docker build -t myapp:latest -f dist/Dockerfile .
docker push myapp:latest

# Deploy infrastructure
cd terraform
terraform apply
```

### With CI/CD

GitHub Actions example:

```yaml
# .github/workflows/test.yml
name: Test

on: [push]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Install Realm
        run: cargo install realmenv

      - name: Initialize
        run: realm init -y

      - name: Activate
        run: source .venv/bin/activate

      - name: Install deps
        run: |
          cd frontend && bun install
          cd backend && bun install

      - name: Start services
        run: |
          realm start &
          sleep 10

      - name: Run tests
        run: bun test

      - name: E2E tests
        run: bun run test:e2e
```

## Custom Commands

### Alias Common Operations

Add to `.bashrc` or `.zshrc`:

```bash
# Quick start
alias rs='realm start'

# Quick stop
alias rx='realm stop'

# Activate current project
alias ra='source .venv/bin/activate'

# Init with defaults
alias ri='realm init -y && source .venv/bin/activate'

# Bundle and deploy
alias rd='realm bundle && cd dist && ./deploy.sh'
```

### Shell Functions

More complex helpers:

```bash
# Initialize and setup project
realm-new() {
  realm init "$1" --template="${2:-react-express}"
  cd "$1"
  source .venv/bin/activate
  cd project/frontend && bun install
  cd ../backend && bun install
  cd ../..
  echo "Ready! Run 'realm start'"
}

# Quick restart
realm-restart() {
  realm stop
  realm start
}

# Clean and rebuild
realm-clean() {
  deactivate
  rm -rf .venv
  realm init -y
  source .venv/bin/activate
}
```

Usage:
```bash
realm-new myapp react-express
realm-restart
realm-clean
```

## Security Hardening

### Read-Only Environments

Prevent modifications:

```bash
chmod -R a-w .venv
realm start  # Can't modify environment
```

### Sandboxed Processes

Use containers for isolation:

```yaml
processes:
  backend:
    command: "docker run --rm --network none mybackend"
    port: 4001
    routes: ["/api/*"]
```

### Secret Scanning

Check for leaked secrets:

```bash
#!/bin/bash
# scripts/check-secrets.sh

echo "Scanning for secrets..."

# Check for common secret patterns
grep -r "password\s*=\s*['\"][^'\"]\+" . --exclude-dir=node_modules
grep -r "api_key\s*=\s*['\"][^'\"]\+" . --exclude-dir=node_modules
grep -r "secret\s*=\s*['\"][^'\"]\+" . --exclude-dir=node_modules

echo "Scan complete!"
```

## Best Practices

### Project Documentation

Document your realm setup:

```markdown
# Development Setup

## Prerequisites
- Realm 1.2.0+
- Bun 1.1.34+

## Quick Start
\`\`\`bash
realm init -y
source .venv/bin/activate
bun install  # In frontend/ and backend/
realm start
\`\`\`

## Environment Variables
See `.env.example` for required variables.

## Processes
- **frontend**: React app on port 4000
- **backend**: Express API on port 4001
- **database**: PostgreSQL on port 5432
```

### Version Control

`.gitignore`:
```gitignore
# Realm
.venv/

# Environment
.env
.env.local

# Dependencies
node_modules/
__pycache__/

# Build
dist/
build/
```

Commit:
```gitignore
# Do commit
realm.yml
.env.example
package.json
```

### Team Collaboration

Setup script for new team members:

```bash
#!/bin/bash
# scripts/setup.sh

echo "Setting up development environment..."

# Install realm if needed
if ! command -v realm &> /dev/null; then
  echo "Installing realm..."
  cargo install realmenv
fi

# Initialize
realm init -y

# Activate
source .venv/bin/activate

# Install dependencies
echo "Installing dependencies..."
cd project
cd frontend && bun install
cd ../backend && bun install
cd ../..

# Setup environment
cp .env.example .env
echo "Edit .env with your settings"

echo "Setup complete! Run 'realm start' to begin."
```

## Next Steps

- [Deployment](deployment.md) - Production deployment
- [Troubleshooting](troubleshooting.md) - Common issues
- [Configuration](configuration.md) - realm.yml reference
