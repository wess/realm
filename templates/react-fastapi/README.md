# React + FastAPI Full-Stack Template

React frontend with FastAPI (Python) backend.

## Structure

```
.
├── frontend/        # React app with Vite
├── backend/         # FastAPI server
└── realm.yml        # Realm configuration
```

## Getting Started

```bash
# Install frontend dependencies
cd frontend && bun install

# Install backend dependencies
cd ../backend && pip install -r requirements.txt

# Start development (from project root)
realm start
```

Visit http://localhost:8000

## What's Running

- **Frontend**: http://localhost:8000/ (Vite dev server on port 4000)
- **Backend**: http://localhost:8000/api/* (FastAPI server on port 4001)
- **Proxy**: http://localhost:8000 (Realm proxy)

## API Endpoints

- `GET /api/health` - Health check endpoint
- `GET /api/hello` - Hello message endpoint

## Building for Production

```bash
# Build frontend
cd frontend
bun run build

# Backend runs directly from source
cd backend
uvicorn main:app --host 0.0.0.0 --port 4001
```
