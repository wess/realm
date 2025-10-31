# Realm Templates

File-based project templates for `realm init`.

## Structure

Each template is a complete directory structure that gets copied to the target location:

```
templates/
├── react-express/       # React frontend + Express backend
│   ├── frontend/
│   ├── backend/
│   ├── realm.yml
│   └── README.md
├── nextjs/             # Next.js with App Router
├── vue-express/        # Vue 3 + Express
├── svelte-fastify/     # SvelteKit + Fastify
└── react-fastapi/      # React + FastAPI (Python)
```

## Creating Templates

1. Create a new directory under `templates/`
2. Add all project files/folders
3. Include a `realm.yml` with process configuration
4. Add a README explaining the template structure

Templates are copied as-is during `realm init`, preserving directory structure and file contents.

## Template Guidelines

- Use relative paths in realm.yml
- Include package.json/requirements.txt with exact versions
- Add .gitignore with common patterns
- Document dev server ports and routes
- Keep templates minimal but functional

## Available Templates

- **react-express**: React (Vite) + Express, Bun runtime
- More templates coming soon via community contributions
