# Next.js Full-Stack Template

Next.js 14 with App Router, powered by Bun.

## Structure

```
.
├── app/
│   ├── api/health/    # API routes
│   ├── layout.tsx     # Root layout
│   └── page.tsx       # Home page
└── realm.yml          # Realm configuration
```

## Getting Started

```bash
# Install dependencies
bun install

# Start development
realm start
```

Visit http://localhost:8000

## What's Running

- **Next.js**: http://localhost:8000 (dev server on port 3000)
- **Proxy**: http://localhost:8000 (Realm proxy)

## API Routes

- `GET /api/health` - Health check endpoint

## Building for Production

```bash
bun run build
bun run start
```
