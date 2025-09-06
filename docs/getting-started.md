# Getting Started with Realm

Realm is a full-stack development environment CLI that simplifies modern web development by providing isolated environments with built-in process management and proxy routing.

## Quick Start

### 1. Initialize a New Project

Create a new realm environment with a template:

```bash
# Initialize with React + Express template
realm init --template react

# Or with Node.js instead of Bun
realm init --runtime node --template react

# Or start from scratch
realm init
```

### 2. Activate the Environment

```bash
source .venv/bin/activate
```

### 3. Start Development

```bash
realm start
```

Your application is now running with:
- Frontend: http://localhost:8080 (proxied)
- Backend API: http://localhost:8080/api (proxied)
- Direct access: http://localhost:3000 (frontend), http://localhost:3001 (backend)

## Core Concepts

### Virtual Environments

Like Python's virtualenv, realm creates isolated environments:

```bash
realm init myproject    # Create environment
source myproject/bin/activate  # Activate environment
realm start            # Start processes
```

### Process Management

Define processes in `realm.yml`:

```yaml
processes:
  frontend:
    cmd: npm run dev
    port: 3000
    routes: ["/"]
  
  backend:
    cmd: npm run start
    port: 3001
    routes: ["/api/*"]
```

### Intelligent Proxy

The built-in proxy routes requests based on URL patterns:
- `/api/*` → backend server
- `/` → frontend server
- Automatic CORS headers
- WebSocket support

### Multi-Runtime Support

Realm supports both Bun and Node.js with version management:

```bash
realm init --runtime bun@1.0.0
realm init --runtime node@18
```

## Installation

### Prerequisites

- Rust (for building from source)
- Git

### From Source

```bash
git clone <repository-url>
cd realm
cargo build --release
```

Add the binary to your PATH or use `cargo install --path .`.

## Available Templates

Realm includes built-in templates for popular stacks:

- **react** - React frontend with Express backend
- **svelte** - SvelteKit with Fastify backend  
- **vue** - Vue.js with Express backend
- **nextjs** - Next.js full-stack application

View all templates:

```bash
realm templates list
```

## Configuration

### Basic realm.yml

```yaml
env:
  NODE_ENV: development
  
proxy_port: 8080

processes:
  app:
    cmd: npm run dev
    port: 3000
    routes: ["/"]
```

### Advanced Configuration

```yaml
env:
  NODE_ENV: development
  LOG_LEVEL: debug

env_file: .env

proxy_port: 8080

processes:
  frontend:
    cmd: npm run dev
    port: 3000
    routes: ["/", "/static/*"]
    working_dir: ./frontend
    
  backend:
    cmd: npm run start:dev
    port: 3001
    routes: ["/api/*", "/health"]
    working_dir: ./backend
    env:
      PORT: "3001"
      DATABASE_URL: postgresql://localhost/myapp
    dependencies: [database]
    
  database:
    cmd: docker run --rm -p 5432:5432 postgres:15
    port: 5432
    routes: []
    env:
      POSTGRES_DB: myapp
```

## Common Workflows

### Development

```bash
# Start everything
realm start

# Start only proxy (manage processes manually)
realm proxy

# Stop everything
realm stop
```

### Deployment

```bash
# Create deployment bundle
realm bundle

# Deploy with Docker
cd deploy/
docker-compose up -d
```

### Template Management

```bash
# Create custom template from current project
realm create --template my-stack

# Use custom template
realm init --template my-stack newproject
```

## Directory Structure

After `realm init`:

```
.venv/                  # Realm environment
├── bin/activate       # Activation script
├── runtimes/          # Installed runtime versions
└── lib/realm/         # Realm libraries

realm.yml              # Configuration file
project/               # Your application (if using template)
├── frontend/
├── backend/
└── shared/
```

## Environment Variables

Variables are loaded in order:
1. System environment
2. `realm.yml` env section
3. File specified in `env_file`
4. Process-specific `env` section

Example `.env` file:

```env
DATABASE_URL=postgresql://localhost/myapp_dev
API_KEY=dev_key_123
LOG_LEVEL=debug
```

## Process Dependencies

Ensure processes start in the correct order:

```yaml
processes:
  backend:
    cmd: npm start
    dependencies: [database, redis]
    
  database:
    cmd: docker run postgres
    
  redis:
    cmd: docker run redis
```

## Troubleshooting

### Common Issues

**"Not in an activated realm environment"**
```bash
source .venv/bin/activate
```

**"Port already in use"**
- Check if other processes are using the ports
- Update port numbers in `realm.yml`

**"Process failed to start"**
- Check the process logs
- Ensure all dependencies are installed
- Verify the working directory exists

**"502 Bad Gateway"**
- The target process isn't running
- Check if the process is listening on the expected port

### Debug Mode

Add debug information to your configuration:

```yaml
env:
  DEBUG: "realm:*"
  LOG_LEVEL: debug
```

### Health Checks

Use the built-in health endpoint:

```bash
curl http://localhost:8080/health
```

## Best Practices

### Project Organization

- Keep frontend and backend in separate directories
- Use shared directories for common code (types, utilities)
- Include proper `.gitignore` files

### Configuration

- Use environment variables for configuration
- Keep sensitive data in `.env` files (not committed)
- Document environment variables in a `.env.example` file

### Development

- Use the proxy for all development (don't access services directly)
- Set up proper error boundaries and health checks
- Use TypeScript for better development experience

### Deployment

- Always test with `realm bundle` before deploying
- Use proper Docker health checks
- Set up monitoring and logging in production

## Next Steps

1. **Explore Templates**: Try different built-in templates to see various stack configurations
2. **Custom Templates**: Create templates for your common project structures
3. **Production Deployment**: Use `realm bundle` to create production-ready containers
4. **Integration**: Integrate realm into your existing CI/CD pipelines

## Getting Help

- Use `realm --help` for command information
- Read man pages: `man realm`, `man realm-init`, etc.
- Check configuration reference: `man realm.yml`

For detailed command documentation, see the individual man pages for each command.