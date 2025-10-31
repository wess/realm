# Proxy Server

Realm's built-in HTTP proxy routes requests to your services intelligently.

## Why a Proxy?

### Without a Proxy

```
Frontend: http://localhost:4000
Backend:  http://localhost:4001

Problems:
- CORS configuration needed
- Different ports to remember
- No production-like setup
- Multiple URLs to manage
```

### With Realm's Proxy

```
Everything: http://localhost:8000

Benefits:
- No CORS issues (same origin)
- Single entry point
- Production-like routing
- Automatic WebSocket upgrade
```

## How It Works

The proxy listens on `proxy_port` and forwards requests based on route patterns.

```yaml
proxy_port: 8000

processes:
  frontend:
    port: 4000
    routes: ["/"]

  backend:
    port: 4001
    routes: ["/api/*"]
```

Request flow:
- `http://localhost:8000/` → frontend:4000
- `http://localhost:8000/about` → frontend:4000
- `http://localhost:8000/api/users` → backend:4001

## Route Matching

### Pattern Types

**Exact match**:
```yaml
routes: ["/health"]
```
- Matches `/health` only
- Doesn't match `/health/status`

**Wildcard match**:
```yaml
routes: ["/api/*"]
```
- Matches `/api/users`, `/api/products/123`, etc.
- Doesn't match `/api` (needs content after `/api/`)

**Catch-all**:
```yaml
routes: ["/"]
```
- Matches anything not matched by other routes
- Should be lowest priority

### Priority

Routes are evaluated from most to least specific:

1. **Exact matches** - `/health`, `/api/status`
2. **Specific wildcards** - `/api/v1/*`
3. **General wildcards** - `/api/*`
4. **Catch-all** - `/`

### Example

```yaml
processes:
  health:
    port: 4010
    routes: ["/health"]  # Priority 1: Exact

  api_v1:
    port: 4002
    routes: ["/api/v1/*"]  # Priority 2: Specific wildcard

  api:
    port: 4001
    routes: ["/api/*"]  # Priority 3: General wildcard

  frontend:
    port: 4000
    routes: ["/"]  # Priority 4: Catch-all
```

Routing:
- `/health` → health:4010
- `/api/v1/users` → api_v1:4002
- `/api/products` → api:4001
- `/about` → frontend:4000

## Configuration

### proxy_port

Port the proxy listens on.

```yaml
proxy_port: 8000
```

**Default**: 8000
**Range**: 1024-65535

Access your app at `http://localhost:8000`.

### Process Routes

Define routes for each process:

```yaml
processes:
  frontend:
    command: "bun run dev"
    port: 4000
    routes: ["/", "/assets/*"]

  backend:
    command: "bun run server"
    port: 4001
    routes: ["/api/*", "/health"]
```

## Request Handling

### HTTP Methods

All HTTP methods are supported:
- GET
- POST
- PUT
- PATCH
- DELETE
- OPTIONS
- HEAD

Example:
```bash
# GET request
curl http://localhost:8000/api/users

# POST request
curl -X POST http://localhost:8000/api/users -d '{"name":"Alice"}'

# PUT request
curl -X PUT http://localhost:8000/api/users/1 -d '{"name":"Bob"}'
```

### Headers

All headers are forwarded:
```yaml
Request:
  Host: localhost:8000
  User-Agent: curl/7.84.0
  Authorization: Bearer token123

Forwarded to backend:
  Host: localhost:4001
  User-Agent: curl/7.84.0
  Authorization: Bearer token123
```

### Query Parameters

Query strings are preserved:
```bash
# Request
http://localhost:8000/api/users?page=2&limit=10

# Forwarded
http://localhost:4001/api/users?page=2&limit=10
```

### Request Body

POST/PUT bodies are forwarded:
```bash
curl -X POST http://localhost:8000/api/users \
  -H "Content-Type: application/json" \
  -d '{"name":"Alice","email":"alice@example.com"}'

# Forwarded with same body
```

## WebSocket Support

The proxy automatically upgrades WebSocket connections.

### How It Works

1. Client sends HTTP Upgrade request
2. Proxy detects `Upgrade: websocket` header
3. Proxy upgrades connection
4. Bidirectional WebSocket tunnel established

### Example: Vite HMR

Vite dev server uses WebSockets for Hot Module Replacement:

```yaml
processes:
  frontend:
    command: "bun run dev"
    port: 4000
    routes: ["/"]
```

Vite connects:
```javascript
// Browser automatically connects to:
ws://localhost:8000/@vite/client

// Proxy forwards to:
ws://localhost:4000/@vite/client
```

No configuration needed - it just works.

### Example: WebSocket API

Backend WebSocket endpoint:

```yaml
processes:
  backend:
    command: "bun run server"
    port: 4001
    routes: ["/ws", "/api/*"]
```

Client code:
```javascript
// Connect through proxy
const ws = new WebSocket('ws://localhost:8000/ws');

// Proxied to backend:4001/ws
ws.onmessage = (event) => {
  console.log(event.data);
};
```

## CORS Handling

The proxy automatically adds CORS headers for development.

### Headers Added

```
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: GET, POST, PUT, DELETE, PATCH, OPTIONS
Access-Control-Allow-Headers: Content-Type, Authorization, X-Requested-With
Access-Control-Max-Age: 86400
```

### Preflight Requests

OPTIONS requests are handled automatically:

```bash
# Preflight
curl -X OPTIONS http://localhost:8000/api/users \
  -H "Access-Control-Request-Method: POST"

# Response includes CORS headers
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: GET, POST, PUT, DELETE, PATCH, OPTIONS
```

### Production

CORS headers are for development only. In production (using nginx), configure CORS appropriately:

```nginx
location /api/ {
  add_header Access-Control-Allow-Origin "$http_origin" always;
  add_header Access-Control-Allow-Methods "GET, POST, PUT, DELETE" always;
}
```

## Error Handling

### 502 Bad Gateway

When the target process isn't running or not listening:

```yaml
processes:
  backend:
    port: 4001
    routes: ["/api/*"]
```

If backend isn't listening on 4001:
```
GET /api/users
→ 502 Bad Gateway
```

**Solution**: Ensure process is running and listening on configured port.

### 404 Not Found

When no route matches:

```yaml
processes:
  backend:
    routes: ["/api/*"]
```

Request to `/graphql`:
```
GET /graphql
→ 404 Not Found
```

**Solution**: Add a catch-all or additional routes.

### Connection Refused

If proxy can't reach the backend:
```
Error: Connection refused to localhost:4001
```

**Solutions**:
- Check process is running: `realm start`
- Verify port in realm.yml matches process
- Check firewall settings

## Common Patterns

### SPA with API

Single-page app with backend API:

```yaml
proxy_port: 8000

processes:
  frontend:
    port: 4000
    routes: ["/"]  # Serves SPA, catch-all

  backend:
    port: 4001
    routes: ["/api/*"]  # API routes
```

- `/` → SPA (index.html)
- `/about` → SPA (client-side routing)
- `/api/users` → Backend API

### Static Assets

Separate routes for static files:

```yaml
processes:
  frontend:
    port: 4000
    routes: ["/", "/assets/*", "/static/*"]

  backend:
    port: 4001
    routes: ["/api/*"]
```

- `/` → Frontend
- `/assets/logo.png` → Frontend static files
- `/static/style.css` → Frontend static files
- `/api/*` → Backend

### Multiple APIs

Route different API versions:

```yaml
processes:
  frontend:
    port: 4000
    routes: ["/"]

  api_v1:
    port: 4001
    routes: ["/api/v1/*"]

  api_v2:
    port: 4002
    routes: ["/api/v2/*"]

  graphql:
    port: 4003
    routes: ["/graphql"]
```

### Microservices

```yaml
processes:
  frontend:
    port: 4000
    routes: ["/"]

  users_api:
    port: 4001
    routes: ["/api/users/*"]

  products_api:
    port: 4002
    routes: ["/api/products/*"]

  orders_api:
    port: 4003
    routes: ["/api/orders/*"]
```

Each service handles its domain.

### Admin Dashboard

Separate admin interface:

```yaml
processes:
  web:
    port: 4000
    routes: ["/"]

  admin:
    port: 4010
    routes: ["/admin/*"]

  backend:
    port: 4001
    routes: ["/api/*"]
```

- `/` → Public web app
- `/admin/*` → Admin dashboard
- `/api/*` → Shared backend

## Performance

### Connection Pooling

The proxy maintains persistent connections to backends when possible.

### Timeouts

Request timeout: 30 seconds (default)

For long-running requests, use streaming or WebSockets.

### Load Balancing

Realm doesn't support load balancing. Each route maps to one process.

For load balancing, use multiple realm instances or a dedicated load balancer.

## Direct Access

You can still access services directly:

```yaml
proxy_port: 8000

processes:
  frontend:
    port: 4000
    routes: ["/"]

  backend:
    port: 4001
    routes: ["/api/*"]
```

Access:
- Via proxy: `http://localhost:8000/`
- Direct frontend: `http://localhost:4000/`
- Direct backend: `http://localhost:4001/`

Useful for debugging, but prefer using the proxy for development.

## Production Deployment

In production, Realm generates nginx configuration:

```bash
realm bundle
```

Creates `dist/nginx.conf`:

```nginx
upstream frontend {
  server localhost:4000;
}

upstream backend {
  server localhost:4001;
}

server {
  listen 8000;

  location / {
    proxy_pass http://frontend;
  }

  location /api/ {
    proxy_pass http://backend;
  }
}
```

Same routing logic, production-ready server.

## Debugging

### Check Routes

View your route configuration:

```bash
cat realm.yml | grep -A 5 processes
```

### Test with curl

```bash
# Test specific routes
curl -v http://localhost:8000/
curl -v http://localhost:8000/api/health
curl -v http://localhost:8000/assets/logo.png
```

### Check Process Ports

Verify processes are listening:

```bash
# macOS/Linux
lsof -i :4000
lsof -i :4001

# All platforms
curl http://localhost:4000/
curl http://localhost:4001/api/health
```

### Proxy Logs

Realm shows proxy activity:

```
[proxy] → GET / → frontend:4000
[proxy] → GET /api/users → backend:4001
[proxy] ← 200 GET /
[proxy] ← 200 GET /api/users
```

## Best Practices

### Use the Proxy

Always develop through the proxy:
```javascript
// Good
fetch('http://localhost:8000/api/users');

// Avoid
fetch('http://localhost:4001/api/users');
```

### Specific Routes First

Most specific routes should come first in your mental model:

```yaml
# Define specific routes first (mentally)
processes:
  health:
    routes: ["/health"]  # Specific

  api:
    routes: ["/api/*"]  # Less specific

  frontend:
    routes: ["/"]  # Catch-all
```

### Separate Concerns

Different processes for different concerns:

```yaml
# Good
processes:
  frontend:
    routes: ["/"]

  backend:
    routes: ["/api/*"]

# Avoid mixing concerns
processes:
  app:
    routes: ["/", "/api/*"]  # Handles everything
```

### Document Routes

Comment your routes:

```yaml
processes:
  frontend:
    routes: ["/"]  # SPA with client-side routing

  backend:
    routes: ["/api/*"]  # REST API

  graphql:
    routes: ["/graphql"]  # GraphQL endpoint
```

## Limitations

### No Load Balancing

One route → one process. Can't split load across multiple instances.

### No URL Rewriting

Routes are matched as-is. No regex or rewriting:

```yaml
# Can't do:
routes: ["/api/*"]  # Strip /api prefix
→ Forward as /*
```

### No Rate Limiting

No built-in rate limiting. Add to your services if needed.

### No Caching

No HTTP caching. Services handle caching themselves.

## Next Steps

- [Process Management](processes.md) - Configure services
- [Configuration](configuration.md) - Full realm.yml reference
- [Deployment](deployment.md) - Production nginx setup
