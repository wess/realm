# Templates

Templates are pre-configured project structures that help you bootstrap new applications quickly.

## Built-in Templates

Realm includes templates for popular full-stack combinations.

### react-express

React frontend with Express backend.

**Runtime**: Bun or Node.js

**Structure**:
```
project/
├── frontend/          # React with Vite
│   ├── src/
│   ├── package.json
│   └── vite.config.ts
├── backend/           # Express API
│   ├── server.js
│   └── package.json
└── realm.yml
```

**Usage**:
```bash
realm init myapp --template=react-express --runtime=bun
cd myapp
source .venv/bin/activate
realm start
```

**Routes**:
- `/` → React frontend (Vite dev server)
- `/api/*` → Express backend

### react-fastapi

React frontend with FastAPI backend.

**Runtime**: Python

**Structure**:
```
project/
├── frontend/          # React with Vite
│   ├── src/
│   ├── package.json
│   └── vite.config.ts
├── backend/           # FastAPI
│   ├── main.py
│   └── requirements.txt
└── realm.yml
```

**Usage**:
```bash
realm init myapp --template=react-fastapi --runtime=python@3.12
cd myapp
source .venv/bin/activate
pip install -r backend/requirements.txt
realm start
```

**Routes**:
- `/` → React frontend
- `/api/*` → FastAPI backend
- `/docs` → FastAPI Swagger docs

### vue-express

Vue 3 frontend with Express backend.

**Runtime**: Bun or Node.js

**Structure**:
```
project/
├── frontend/          # Vue 3 with Vite
│   ├── src/
│   ├── package.json
│   └── vite.config.ts
├── backend/           # Express API
│   ├── server.js
│   └── package.json
└── realm.yml
```

**Usage**:
```bash
realm init myapp --template=vue-express --runtime=bun
cd myapp
source .venv/bin/activate
realm start
```

### svelte-fastify

SvelteKit frontend with Fastify backend.

**Runtime**: Bun or Node.js

**Structure**:
```
project/
├── frontend/          # SvelteKit
│   ├── src/
│   ├── package.json
│   └── svelte.config.js
├── backend/           # Fastify API
│   ├── server.js
│   └── package.json
└── realm.yml
```

**Usage**:
```bash
realm init myapp --template=svelte-fastify --runtime=bun
cd myapp
source .venv/bin/activate
realm start
```

### nextjs

Next.js 14 full-stack application.

**Runtime**: Bun or Node.js

**Structure**:
```
project/
├── app/               # Next.js 14 app directory
│   ├── page.tsx
│   └── api/           # API routes
├── package.json
├── next.config.js
└── realm.yml
```

**Usage**:
```bash
realm init myapp --template=nextjs --runtime=bun
cd myapp
source .venv/bin/activate
realm start
```

**Routes**:
- `/*` → Next.js handles all routes (frontend + API)

## Using Templates

### Interactive Selection

```bash
realm init
# → Use a project template?:
#     No template
#     React + Express
#     React + FastAPI
#     Vue + Express
#     Svelte + Fastify
#     Next.js
```

### Command-Line

```bash
realm init myapp --template=react-express --runtime=bun
```

### List Available Templates

```bash
realm templates list
```

Output:
```
📄 Available templates:
   • react-express (built-in)
   • react-fastapi (built-in)
   • vue-express (built-in)
   • svelte-fastify (built-in)
   • nextjs (built-in)
   • my-stack (custom)
```

## Template Variables

Templates can define custom variables that users provide during initialization. This allows templates to be more flexible and generate personalized project structures.

### Using Variables in Templates

Pass variables via CLI:

```bash
realm init myapp --template=react-express \
  --var project_name=myapp \
  --var author="John Doe" \
  --var description="My awesome app"
```

Interactive prompts (if variables not provided):

```bash
realm init --template=react-express
# Prompts:
# > Project name? (myapp)
# > Author? ()
# > Description? (A React and Express application)
```

Skip prompts with defaults:

```bash
realm init myapp --template=react-express --yes
# Uses default values for all variables
```

### Defining Variables in Templates

Create a `template.yaml` in your template directory:

```yaml
name: my-template
description: My custom template
variables:
  - name: project_name
    prompt: "Project name"
    default: "{{directory_name}}"  # Uses directory name by default

  - name: author
    prompt: "Author name"
    default: ""

  - name: description
    prompt: "Project description"
    default: "A full-stack application"

  - name: api_port
    prompt: "API port"
    default: "3001"

  - name: database_url
    prompt: "Database URL"
    default: "postgresql://localhost/myapp"
```

### Using Variables in Template Files

Template files support Tera syntax for variable substitution:

**package.json**:
```json
{
  "name": "{{project_name}}-frontend",
  "version": "1.0.0",
  "author": "{{author}}",
  "description": "{{description}}"
}
```

**README.md**:
```markdown
# {{project_name}}

{{description}}

## Author

{{author}}

## Development

Start the development server on port {{api_port}}:
```

**.env.example**:
```env
PROJECT_NAME={{project_name}}
DATABASE_URL={{database_url}}
API_PORT={{api_port}}
```

**realm.yml**:
```yaml
processes:
  backend:
    command: "bun run dev"
    port: {{api_port}}
    routes: ["/api/*"]
```

### Variable Defaults

The `{{directory_name}}` placeholder is replaced with the actual directory name:

```yaml
variables:
  - name: project_name
    prompt: "Project name"
    default: "{{directory_name}}"
```

When running:
```bash
realm init myapp --template=my-template
```

The `project_name` variable will default to `myapp`.

### Built-in Template Variables

All built-in templates support these variables:

- **project_name**: Name of the project (defaults to directory name)
- **author**: Author name (defaults to empty string)
- **description**: Project description (defaults to template description)

### Variable Best Practices

**Keep Variables Minimal**

Only define variables that genuinely need customization:

```yaml
# Good - essential customization
variables:
  - name: project_name
    prompt: "Project name"
    default: "{{directory_name}}"

  - name: api_port
    prompt: "API port"
    default: "3001"

# Avoid - too many variables overwhelms users
variables:
  - name: use_typescript
  - name: use_eslint
  - name: use_prettier
  - name: ui_library
  # ... (too many options)
```

**Provide Sensible Defaults**

Always provide defaults so users can skip prompts:

```yaml
variables:
  - name: database_url
    prompt: "Database URL"
    default: "postgresql://localhost/{{project_name}}"  # ✅ Good default

  - name: api_key
    prompt: "API Key"
    default: ""  # ✅ Empty string for sensitive data
```

**Use Clear Prompts**

Make prompts self-explanatory:

```yaml
# Good
- name: api_port
  prompt: "API server port"
  default: "3001"

# Better
- name: api_port
  prompt: "Port for API server (3000-9999)"
  default: "3001"
```

### Example: Full Template with Variables

**template.yaml**:
```yaml
name: fullstack-template
description: Full-stack TypeScript application
variables:
  - name: project_name
    prompt: "Project name"
    default: "{{directory_name}}"

  - name: author
    prompt: "Author"
    default: ""

  - name: description
    prompt: "Description"
    default: "A modern full-stack application"

  - name: frontend_port
    prompt: "Frontend port"
    default: "4000"

  - name: backend_port
    prompt: "Backend port"
    default: "4001"
```

**frontend/package.json**:
```json
{
  "name": "{{project_name}}-frontend",
  "version": "1.0.0",
  "author": "{{author}}",
  "description": "{{description}} - Frontend"
}
```

**backend/package.json**:
```json
{
  "name": "{{project_name}}-backend",
  "version": "1.0.0",
  "author": "{{author}}",
  "description": "{{description}} - Backend"
}
```

**realm.yml**:
```yaml
proxy_port: 8000

processes:
  frontend:
    command: "bun run dev"
    port: {{frontend_port}}
    routes: ["/"]
    working_directory: "frontend"

  backend:
    command: "bun run dev"
    port: {{backend_port}}
    routes: ["/api/*"]
    working_directory: "backend"
```

**Usage**:
```bash
# With all variables
realm init myapp --template=fullstack-template \
  --var project_name=myapp \
  --var author="Alice" \
  --var frontend_port=3000 \
  --var backend_port=3001

# Interactive (prompts for each variable)
realm init myapp --template=fullstack-template

# With defaults (no prompts)
realm init myapp --template=fullstack-template --yes
```

## Custom Templates

Create templates from your own projects.

### Creating a Template

From your project directory:

```bash
realm create --template=my-stack
```

This copies your project to `~/.realm/templates/my-stack/`.

### Template Structure

A template should include:

```
my-stack/
├── realm.yml          # Required: Process configuration
├── frontend/          # Optional: Frontend code
├── backend/           # Optional: Backend code
├── .gitignore        # Recommended
├── README.md         # Recommended
└── .env.example      # Recommended
```

### What Gets Copied

Realm copies everything except:
- `.venv/` - Realm environment
- `node_modules/` - Node dependencies
- `__pycache__/` - Python cache
- `.git/` - Git repository
- `dist/`, `build/` - Build artifacts
- `*.log` - Log files

### Using Custom Templates

```bash
realm init newproject --template=my-stack
```

### Template realm.yml

Your template's `realm.yml` should use generic paths:

```yaml
proxy_port: 8000

env:
  NODE_ENV: development

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

### Template README

Include setup instructions:

```markdown
# My Stack Template

## Setup

1. Install dependencies:
   ```bash
   cd frontend && bun install
   cd backend && bun install
   ```

2. Configure environment:
   ```bash
   cp .env.example .env
   # Edit .env with your values
   ```

3. Start development:
   ```bash
   realm start
   ```

## Project Structure

- `frontend/` - React app with TypeScript
- `backend/` - Express API with TypeScript
- `shared/` - Shared types and utilities
```

## Template Best Practices

### Include .env.example

Document required environment variables:

```env
# .env.example
DATABASE_URL=postgresql://localhost/myapp
API_KEY=your_api_key_here
JWT_SECRET=your_jwt_secret
LOG_LEVEL=info
```

### Add .gitignore

Prevent committing sensitive files:

```gitignore
# .gitignore
.env
.venv/
node_modules/
__pycache__/
*.log
dist/
build/
```

### Keep Dependencies Minimal

Start with minimal dependencies. Users can add more as needed.

### Use TypeScript

TypeScript provides better developer experience:

```
frontend/
├── src/
│   ├── types/        # Shared types
│   └── components/
└── tsconfig.json

backend/
├── src/
│   ├── types/        # Shared types
│   └── routes/
└── tsconfig.json
```

### Include Tests

Add basic test setup:

```
frontend/
├── src/
│   └── components/
│       ├── Button.tsx
│       └── Button.test.tsx
└── vitest.config.ts

backend/
├── src/
│   └── routes/
│       ├── users.ts
│       └── users.test.ts
└── vitest.config.ts
```

### Document Scripts

In package.json:

```json
{
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "test": "vitest",
    "lint": "eslint .",
    "typecheck": "tsc --noEmit"
  }
}
```

## Example: Custom Template

### Project Structure

```
my-fullstack-template/
├── realm.yml
├── .gitignore
├── .env.example
├── README.md
├── frontend/
│   ├── src/
│   │   ├── App.tsx
│   │   ├── types/
│   │   └── components/
│   ├── package.json
│   ├── tsconfig.json
│   └── vite.config.ts
├── backend/
│   ├── src/
│   │   ├── server.ts
│   │   ├── types/
│   │   └── routes/
│   ├── package.json
│   ├── tsconfig.json
│   └── vitest.config.ts
└── shared/
    └── types/
        └── api.ts
```

### realm.yml

```yaml
proxy_port: 8000

env:
  NODE_ENV: development
  API_URL: http://localhost:8000/api

env_file: .env

processes:
  frontend:
    command: "bun run dev"
    port: 4000
    routes: ["/", "/assets/*"]
    working_directory: "frontend"
    env:
      VITE_API_URL: "http://localhost:8000/api"

  backend:
    command: "bun run dev"
    port: 4001
    routes: ["/api/*", "/health"]
    working_directory: "backend"
    env:
      PORT: "4001"
```

### README.md

```markdown
# My Full-Stack Template

TypeScript full-stack template with React + Express.

## Stack

- **Frontend**: React 18, TypeScript, Vite, Vitest
- **Backend**: Express, TypeScript, Vitest
- **Shared**: Shared TypeScript types

## Setup

1. Install dependencies:
   ```bash
   cd frontend && bun install
   cd ../backend && bun install
   ```

2. Configure environment:
   ```bash
   cp .env.example .env
   ```

3. Start development:
   ```bash
   realm start
   ```

## Available Scripts

### Frontend
- `bun run dev` - Start dev server
- `bun run build` - Build for production
- `bun run test` - Run tests

### Backend
- `bun run dev` - Start dev server
- `bun run build` - Build for production
- `bun run test` - Run tests
```

### Creating the Template

```bash
cd my-fullstack-template
realm create --template=fullstack-ts

# Now available for use
realm init newproject --template=fullstack-ts
```

## Template Locations

Templates are stored in:

```
~/.realm/
└── templates/
    ├── react-express/      # Built-in
    ├── react-fastapi/      # Built-in
    ├── vue-express/        # Built-in
    ├── svelte-fastify/     # Built-in
    ├── nextjs/             # Built-in
    └── my-stack/           # Custom
```

## Managing Templates

### Delete a Template

```bash
rm -rf ~/.realm/templates/my-stack
```

### Update a Template

```bash
# Recreate from updated project
cd my-stack-project
realm create --template=my-stack

# This overwrites the existing template
```

### Share a Template

Package and share your template:

```bash
# Create archive
cd ~/.realm/templates
tar -czf my-stack.tar.gz my-stack/

# Share my-stack.tar.gz

# Others can extract it
mkdir -p ~/.realm/templates
cd ~/.realm/templates
tar -xzf my-stack.tar.gz
```

## Template Development Tips

### Test Your Template

```bash
# Create template
cd my-project
realm create --template=test-template

# Test it
cd /tmp
realm init test-run --template=test-template
cd test-run
source .venv/bin/activate
realm start

# Verify everything works
```

### Version Control

Keep your template source in git:

```bash
git clone https://github.com/you/my-stack-template
cd my-stack-template

# Create Realm template from it
realm create --template=my-stack

# Update template when source changes
git pull
realm create --template=my-stack
```

### Multi-Runtime Support

Make templates work with multiple runtimes:

```yaml
# realm.yml
processes:
  frontend:
    command: "bun run dev"  # Works with bun or node
    port: 4000
    routes: ["/"]

  backend:
    command: "bun run server"  # Works with bun or node
    port: 4001
    routes: ["/api/*"]
```

Users can choose runtime at init:
```bash
realm init app1 --template=my-stack --runtime=bun
realm init app2 --template=my-stack --runtime=node@20
```

## Next Steps

- [Runtime Management](runtimes.md) - Understand runtime versions
- [Process Management](processes.md) - Configure complex services
- [Deployment](deployment.md) - Deploy templates to production
