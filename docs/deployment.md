# Deployment

Deploy Realm projects to production using Docker.

## Overview

Realm generates production deployment artifacts:

```bash
realm bundle
```

Creates:
```
dist/
├── Dockerfile           # Multi-stage build
├── docker-compose.yml   # Service orchestration
├── nginx.conf          # Reverse proxy
└── deploy.sh           # Deployment script
```

## Quick Deploy

```bash
# Generate artifacts
realm bundle

# Deploy
cd dist
./deploy.sh
```

This builds and starts your services with Docker Compose.

## Generated Files

### Dockerfile

Multi-stage build that:
1. Installs dependencies
2. Builds all services
3. Creates minimal runtime image
4. Sets up nginx reverse proxy

Example:
```dockerfile
# Build stage
FROM node:20-alpine AS build

WORKDIR /app

# Copy source
COPY frontend/ ./frontend/
COPY backend/ ./backend/

# Install and build frontend
WORKDIR /app/frontend
RUN npm install
RUN npm run build

# Install backend
WORKDIR /app/backend
RUN npm install

# Runtime stage
FROM node:20-alpine

# Install nginx
RUN apk add --no-cache nginx

# Copy built assets
COPY --from=build /app/frontend/dist /app/frontend/dist
COPY --from=build /app/backend /app/backend

# Copy nginx config
COPY nginx.conf /etc/nginx/nginx.conf

EXPOSE 8000

CMD ["sh", "-c", "nginx && node /app/backend/server.js"]
```

### docker-compose.yml

Orchestrates services:

```yaml
version: '3.8'

services:
  app:
    build: .
    ports:
      - "8000:8000"
    environment:
      NODE_ENV: production
      DATABASE_URL: postgresql://db:5432/myapp
    depends_on:
      - db

  db:
    image: postgres:15
    environment:
      POSTGRES_DB: myapp
      POSTGRES_USER: user
      POSTGRES_PASSWORD: password
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
```

### nginx.conf

Replicates your realm.yml routing:

```nginx
events {
  worker_connections 1024;
}

http {
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
      proxy_http_version 1.1;
      proxy_set_header Upgrade $http_upgrade;
      proxy_set_header Connection 'upgrade';
      proxy_set_header Host $host;
      proxy_cache_bypass $http_upgrade;
    }

    location /api/ {
      proxy_pass http://backend;
      proxy_http_version 1.1;
      proxy_set_header Host $host;
    }
  }
}
```

### deploy.sh

Deployment script:

```bash
#!/bin/bash
set -e

echo "Building Docker image..."
docker-compose build

echo "Starting services..."
docker-compose up -d

echo "Deployment complete!"
echo "App running at http://localhost:8000"
```

## Deployment Workflow

### 1. Prepare for Production

Ensure production-ready configuration:

```yaml
# realm.yml
proxy_port: 8000

env:
  NODE_ENV: production

env_file: .env.production

processes:
  frontend:
    command: "npm run build && npm run preview"
    port: 4000
    routes: ["/"]

  backend:
    command: "npm start"
    port: 4001
    routes: ["/api/*"]
```

### 2. Generate Bundle

```bash
realm bundle
```

Output:
```
📦 Creating deployment bundle...
   ✓ Generated Dockerfile
   ✓ Generated docker-compose.yml
   ✓ Generated nginx.conf
   ✓ Generated deploy.sh

Deployment bundle created in: dist/
```

### 3. Review Generated Files

Check the generated files:

```bash
cd dist
cat Dockerfile
cat docker-compose.yml
cat nginx.conf
```

Make any necessary adjustments.

### 4. Deploy

#### Local Testing

```bash
./deploy.sh
```

#### Production Server

```bash
# Copy to server
scp -r dist/ user@server:/app/

# SSH to server
ssh user@server

# Deploy
cd /app/dist
./deploy.sh
```

### 5. Verify

```bash
curl http://localhost:8000/
curl http://localhost:8000/api/health
```

## Customizing Deployment

### Modify Dockerfile

Edit `dist/Dockerfile` for custom needs:

```dockerfile
# Add system dependencies
RUN apk add --no-cache python3 py3-pip

# Set up cron jobs
RUN echo "0 * * * * /app/scripts/backup.sh" > /etc/crontabs/root

# Custom entrypoint
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh
ENTRYPOINT ["/entrypoint.sh"]
```

### Modify docker-compose.yml

Add services or configuration:

```yaml
services:
  app:
    build: .
    ports:
      - "8000:8000"
    volumes:
      - ./uploads:/app/uploads  # Persistent uploads
    restart: unless-stopped     # Auto-restart

  db:
    image: postgres:15
    volumes:
      - postgres_data:/var/lib/postgresql/data
    restart: unless-stopped

  redis:
    image: redis:alpine
    restart: unless-stopped

volumes:
  postgres_data:
```

### Modify nginx.conf

Add custom nginx configuration:

```nginx
http {
  # Enable gzip
  gzip on;
  gzip_types text/plain text/css application/json application/javascript;

  # Set up caching
  proxy_cache_path /var/cache/nginx levels=1:2 keys_zone=my_cache:10m;

  server {
    listen 8000;

    # Cache static assets
    location /assets/ {
      proxy_pass http://frontend;
      proxy_cache my_cache;
      expires 1y;
      add_header Cache-Control "public, immutable";
    }

    # API routes
    location /api/ {
      proxy_pass http://backend;

      # Rate limiting
      limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
      limit_req zone=api burst=20;
    }
  }
}
```

## Environment Variables

### Production .env

Create `.env.production`:

```env
NODE_ENV=production
DATABASE_URL=postgresql://db:5432/myapp
REDIS_URL=redis://redis:6379
JWT_SECRET=prod_secret_here
API_KEY=prod_api_key_here
```

### Secrets Management

Don't commit production secrets. Use:

1. **Environment variables** (docker-compose):
```yaml
services:
  app:
    environment:
      - DATABASE_URL=${DATABASE_URL}
      - JWT_SECRET=${JWT_SECRET}
```

2. **Docker secrets**:
```yaml
services:
  app:
    secrets:
      - db_password
      - jwt_secret

secrets:
  db_password:
    file: ./secrets/db_password.txt
  jwt_secret:
    file: ./secrets/jwt_secret.txt
```

3. **External secret managers** (AWS Secrets Manager, HashiCorp Vault)

## Multi-Stage Builds

Optimize Docker image size with multi-stage builds.

### Frontend Build

```dockerfile
# Build stage
FROM node:20-alpine AS frontend-build
WORKDIR /app/frontend
COPY frontend/package*.json ./
RUN npm ci --only=production
COPY frontend/ ./
RUN npm run build

# Runtime stage
FROM nginx:alpine
COPY --from=frontend-build /app/frontend/dist /usr/share/nginx/html
```

### Backend Build

```dockerfile
# Build stage
FROM node:20-alpine AS backend-build
WORKDIR /app/backend
COPY backend/package*.json ./
RUN npm ci --only=production
COPY backend/ ./
RUN npm run build

# Runtime stage
FROM node:20-alpine
WORKDIR /app
COPY --from=backend-build /app/backend/dist ./dist
COPY --from=backend-build /app/backend/node_modules ./node_modules
CMD ["node", "dist/server.js"]
```

## Database Migrations

### Run Migrations on Deploy

Add to `deploy.sh`:

```bash
#!/bin/bash
set -e

echo "Building Docker image..."
docker-compose build

echo "Running database migrations..."
docker-compose run --rm app npm run migrate

echo "Starting services..."
docker-compose up -d
```

### Migration Script

In `package.json`:

```json
{
  "scripts": {
    "migrate": "node scripts/migrate.js"
  }
}
```

## Health Checks

### Docker Health Checks

Add to `Dockerfile`:

```dockerfile
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8000/health || exit 1
```

### docker-compose Health Checks

```yaml
services:
  app:
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8000/health"]
      interval: 30s
      timeout: 3s
      retries: 3
      start_period: 5s

  db:
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U $$POSTGRES_USER"]
      interval: 10s
      timeout: 5s
      retries: 5
```

## Logging

### Centralized Logging

Configure log drivers:

```yaml
services:
  app:
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
```

### External Log Services

```yaml
services:
  app:
    logging:
      driver: "syslog"
      options:
        syslog-address: "tcp://logs.example.com:514"
```

## Monitoring

### Add Prometheus Metrics

Expose metrics endpoint:

```yaml
# realm.yml
processes:
  backend:
    routes: ["/api/*", "/metrics"]
```

### Grafana + Prometheus

```yaml
# docker-compose.yml
services:
  prometheus:
    image: prom/prometheus
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
    ports:
      - "9090:9090"

  grafana:
    image: grafana/grafana
    ports:
      - "3000:3000"
    depends_on:
      - prometheus
```

## Scaling

### Horizontal Scaling

Use Docker Swarm or Kubernetes for scaling.

#### Docker Swarm

```bash
docker swarm init
docker stack deploy -c docker-compose.yml myapp
docker service scale myapp_app=3
```

#### Kubernetes

Convert with Kompose:

```bash
kompose convert -f docker-compose.yml
kubectl apply -f .
kubectl scale deployment app --replicas=3
```

### Load Balancing

Add nginx load balancing:

```nginx
upstream backend {
  server backend1:4001;
  server backend2:4001;
  server backend3:4001;
}

server {
  location /api/ {
    proxy_pass http://backend;
  }
}
```

## CI/CD Integration

### GitHub Actions

```yaml
# .github/workflows/deploy.yml
name: Deploy

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Install Realm
        run: cargo install realmenv

      - name: Generate bundle
        run: realm bundle

      - name: Deploy to server
        run: |
          scp -r dist/ user@server:/app/
          ssh user@server 'cd /app/dist && ./deploy.sh'
```

### GitLab CI

```yaml
# .gitlab-ci.yml
deploy:
  stage: deploy
  script:
    - cargo install realmenv
    - realm bundle
    - scp -r dist/ user@server:/app/
    - ssh user@server 'cd /app/dist && ./deploy.sh'
  only:
    - main
```

## Cloud Platforms

### AWS ECS

Deploy to Elastic Container Service:

```bash
# Build and push image
docker build -t myapp:latest -f dist/Dockerfile .
docker tag myapp:latest 123456789.dkr.ecr.us-east-1.amazonaws.com/myapp:latest
docker push 123456789.dkr.ecr.us-east-1.amazonaws.com/myapp:latest

# Deploy to ECS
aws ecs update-service --cluster mycluster --service myapp --force-new-deployment
```

### Google Cloud Run

```bash
# Build and push
docker build -t gcr.io/myproject/myapp:latest -f dist/Dockerfile .
docker push gcr.io/myproject/myapp:latest

# Deploy
gcloud run deploy myapp --image gcr.io/myproject/myapp:latest --platform managed
```

### DigitalOcean App Platform

```yaml
# .do/app.yaml
name: myapp
services:
  - name: web
    dockerfile_path: dist/Dockerfile
    github:
      repo: user/myapp
      branch: main
    envs:
      - key: NODE_ENV
        value: production
```

## Best Practices

### Minimize Image Size

```dockerfile
# Use alpine images
FROM node:20-alpine

# Multi-stage builds
FROM node:20 AS build
# ... build steps
FROM node:20-alpine
COPY --from=build /app/dist ./dist

# Clean up
RUN npm prune --production
RUN rm -rf /tmp/* /var/cache/apk/*
```

### Security

```dockerfile
# Don't run as root
RUN addgroup -g 1001 -S nodejs
RUN adduser -S nodejs -u 1001
USER nodejs

# Read-only root filesystem
docker run --read-only myapp:latest

# No privileged mode
docker run --security-opt=no-new-privileges myapp:latest
```

### Environment-Specific Configs

```yaml
# docker-compose.prod.yml
services:
  app:
    restart: always
    environment:
      NODE_ENV: production
    deploy:
      replicas: 3
      resources:
        limits:
          cpus: '0.5'
          memory: 512M
```

Deploy:
```bash
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

## Troubleshooting

### Container Won't Start

```bash
# Check logs
docker-compose logs app

# Interactive shell
docker-compose run --rm app sh
```

### Port Conflicts

```bash
# Check what's using port
lsof -i :8000

# Use different port
docker-compose run -p 9000:8000 app
```

### Database Connection Issues

```bash
# Check database
docker-compose logs db

# Test connection
docker-compose exec app sh
ping db
nc -zv db 5432
```

## Next Steps

- [Troubleshooting](troubleshooting.md) - Common deployment issues
- [Advanced Usage](advanced.md) - Complex deployment patterns
- [Configuration](configuration.md) - Optimize for production
