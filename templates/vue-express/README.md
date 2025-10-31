# Vue + Express Full-Stack Template

Vue 3 frontend with Express backend, powered by Bun.

## Structure

```
.
├── frontend/        # Vue 3 app with Vite
├── backend/         # Express API server
└── realm.yml        # Realm configuration
```

## Getting Started

```bash
# Install dependencies
cd frontend && bun install
cd ../backend && bun install

# Start development (from project root)
realm start
```

Visit http://localhost:8000

## What's Running

- **Frontend**: http://localhost:8000/ (Vite dev server on port 4000)
- **Backend**: http://localhost:8000/api/* (Express server on port 4001)
- **Proxy**: http://localhost:8000 (Realm proxy)

## API Endpoints

- `GET /api/health` - Health check endpoint
- `GET /api/users` - Sample users data

## Building for Production

```bash
# Build frontend
cd frontend
bun run build

# Backend runs directly from source
cd backend
bun run server.js
```
