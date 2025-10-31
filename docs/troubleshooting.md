# Troubleshooting

Common issues and solutions when using Realm.

## Installation Issues

### "command not found: realm"

**Problem**: Realm binary not in PATH.

**Solutions**:

1. **Check installation**:
   ```bash
   which realm
   ~/.cargo/bin/realm --version
   ```

2. **Add to PATH** (bash/zsh):
   ```bash
   export PATH="$HOME/.cargo/bin:$PATH"
   # Add to ~/.bashrc or ~/.zshrc
   ```

3. **Use full path**:
   ```bash
   ~/.cargo/bin/realm init
   ```

### "Permission denied" on Install

**Problem**: No write access to installation directory.

**Solution**:
```bash
# For /usr/local/bin
sudo mv realm /usr/local/bin/

# Or install to user directory
mv realm ~/.local/bin/
export PATH="$HOME/.local/bin:$PATH"
```

### Cargo Install Fails

**Problem**: Build errors during `cargo install realmenv`.

**Solutions**:

1. **Update Rust**:
   ```bash
   rustup update
   ```

2. **Check Rust version**:
   ```bash
   rustc --version  # Need 1.75+
   ```

3. **Install dependencies** (Linux):
   ```bash
   # Ubuntu/Debian
   sudo apt install build-essential pkg-config libssl-dev

   # Fedora/RHEL
   sudo dnf install gcc openssl-devel
   ```

## Environment Issues

### "Not in an activated realm environment"

**Problem**: Commands require activated environment.

**Solution**:
```bash
source .venv/bin/activate
```

**Check activation**:
```bash
echo $REALM_ENV  # Should be set
which realm      # Should point to .venv/bin/
```

### Can't Activate Environment

**Problem**: `source .venv/bin/activate` doesn't work.

**Solutions**:

1. **Wrong shell**:
   ```bash
   # Bash/Zsh
   source .venv/bin/activate

   # Fish
   source .venv/bin/activate.fish

   # Windows CMD
   .venv\Scripts\activate.bat

   # Windows PowerShell
   .venv\Scripts\Activate.ps1
   ```

2. **File doesn't exist**:
   ```bash
   ls .venv/bin/activate
   # If missing, re-run: realm init
   ```

3. **Permission issues**:
   ```bash
   chmod +x .venv/bin/activate
   ```

### Deactivate Doesn't Work

**Problem**: `deactivate` command not found.

**Solution**: `deactivate` is only available after activation:
```bash
source .venv/bin/activate
deactivate  # Now works
```

## Runtime Issues

### "Runtime not found"

**Problem**: Specified runtime version doesn't exist.

**Solution**: Check available versions:
```bash
realm list --runtime=bun
realm list --runtime=node
realm list --runtime=python
```

Use a valid version:
```bash
realm init --runtime=python@3.12.6
```

### "Download failed"

**Problem**: Can't download runtime.

**Solutions**:

1. **Check internet**:
   ```bash
   ping github.com
   ping nodejs.org
   ```

2. **Check firewall**: Ensure HTTPS allowed

3. **Try again**: Could be transient network issue

4. **Clear cache**:
   ```bash
   realm cache clear
   realm init --runtime=bun
   ```

5. **Use system runtime**:
   ```bash
   # Install system-wide
   brew install bun  # macOS
   curl -fsSL https://bun.sh/install | bash

   # Use system version
   realm init --runtime=bun  # Uses system if available
   ```

### "Version X.Y.Z not found"

**Problem**: Exact version doesn't exist.

**Solution**: Use major version or check available:
```bash
# Instead of
realm init --runtime=node@20.99.99  # Doesn't exist

# Use major version
realm init --runtime=node@20  # Latest 20.x

# Or check available
realm list --runtime=node
```

### Python Import Errors

**Problem**: `ImportError: No module named 'X'` after activation.

**Solutions**:

1. **Install packages**:
   ```bash
   source .venv/bin/activate
   pip install -r requirements.txt
   ```

2. **Check virtual environment**:
   ```bash
   which python  # Should be .venv/bin/python
   python -c "import sys; print(sys.prefix)"  # Should be .venv
   ```

3. **Reinstall environment**:
   ```bash
   deactivate
   mv .venv .venv.bak
   realm init --runtime=python@3.12
   source .venv/bin/activate
   pip install -r requirements.txt
   ```

## Configuration Issues

### "realm.yml not found"

**Problem**: `realm start` can't find config.

**Solutions**:

1. **Check current directory**:
   ```bash
   ls realm.yml
   pwd
   ```

2. **Run from project root**:
   ```bash
   cd /path/to/project
   realm start
   ```

3. **Create realm.yml**:
   ```bash
   cat > realm.yml << 'EOF'
   proxy_port: 8000

   processes:
     app:
       command: "bun run dev"
       port: 3000
       routes: ["/"]
   EOF
   ```

### "Invalid realm.yml"

**Problem**: Configuration parse error.

**Solutions**:

1. **Check YAML syntax**:
   ```bash
   # Use yamllint if available
   yamllint realm.yml
   ```

2. **Common errors**:
   ```yaml
   # Wrong: Missing quotes
   command: bun run dev

   # Right
   command: "bun run dev"

   # Wrong: Wrong indentation
   processes:
   frontend:
     command: "bun run dev"

   # Right
   processes:
     frontend:
       command: "bun run dev"
   ```

3. **Validate required fields**:
   ```yaml
   # Must have
   proxy_port: 8000

   processes:
     app:
       command: "bun run dev"  # Required
       port: 3000              # Required if routes specified
       routes: ["/"]           # Optional
   ```

### "Dependency not found"

**Problem**: Process references nonexistent dependency.

**Solution**: Fix dependency names:
```yaml
# Wrong
processes:
  backend:
    dependencies: [database]  # But 'database' not defined

# Right
processes:
  backend:
    dependencies: [db]

  db:
    command: "docker run postgres"
```

### "Circular dependency"

**Problem**: Processes depend on each other.

**Solution**: Remove circular dependency:
```yaml
# Wrong
processes:
  frontend:
    dependencies: [backend]

  backend:
    dependencies: [frontend]  # Circular!

# Right
processes:
  frontend:
    dependencies: [backend]

  backend:
    # No dependency on frontend
    command: "bun run server"
```

## Process Issues

### "Port already in use"

**Problem**: Port conflict.

**Solutions**:

1. **Find what's using port**:
   ```bash
   # macOS/Linux
   lsof -i :8000

   # Windows
   netstat -ano | findstr :8000
   ```

2. **Kill process**:
   ```bash
   # macOS/Linux
   kill -9 <PID>

   # Windows
   taskkill /PID <PID> /F
   ```

3. **Use different port**:
   ```yaml
   # realm.yml
   proxy_port: 9000  # Instead of 8000
   ```

### Process Won't Start

**Problem**: Process fails immediately.

**Solutions**:

1. **Check logs**:
   ```bash
   realm start
   # Look for error messages with [process-name] prefix
   ```

2. **Test command directly**:
   ```bash
   cd frontend
   bun run dev  # Test if command works
   ```

3. **Check working directory**:
   ```yaml
   processes:
     frontend:
       command: "bun run dev"
       working_directory: "frontend"  # Must exist
   ```

4. **Check dependencies installed**:
   ```bash
   cd frontend
   bun install  # or npm install
   ```

### "502 Bad Gateway"

**Problem**: Proxy can't reach backend.

**Solutions**:

1. **Check process running**:
   ```bash
   realm start
   # Look for process startup messages
   ```

2. **Test port directly**:
   ```bash
   curl http://localhost:4000/  # Direct to process
   ```

3. **Check port in realm.yml**:
   ```yaml
   processes:
     backend:
       port: 4001  # Must match where backend listens
   ```

4. **Check process health**:
   ```bash
   # Add health endpoint
   curl http://localhost:8000/health
   ```

### Process Keeps Crashing

**Problem**: Process starts then exits.

**Solutions**:

1. **Check logs for errors**:
   ```bash
   realm start
   # Read error messages
   ```

2. **Common causes**:
   - Missing dependencies: `npm install` or `pip install`
   - Wrong working directory: Check `working_directory`
   - Port conflict: Change port
   - Missing environment variables: Check `.env`

3. **Test independently**:
   ```bash
   cd backend
   node server.js  # Run without realm
   ```

## Proxy Issues

### Can't Access http://localhost:8000

**Problem**: Proxy not responding.

**Solutions**:

1. **Check proxy started**:
   ```bash
   realm start
   # Look for: 🌐 Starting proxy server...
   ```

2. **Check port not blocked**:
   ```bash
   curl http://localhost:8000/
   ```

3. **Check firewall**:
   ```bash
   # macOS
   sudo /usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate

   # Linux (ufw)
   sudo ufw status
   ```

### Routes Not Working

**Problem**: Requests go to wrong service.

**Solutions**:

1. **Check route order**:
   ```yaml
   # More specific routes first
   processes:
     health:
       routes: ["/health"]  # Exact match

     api:
       routes: ["/api/*"]  # Wildcard

     frontend:
       routes: ["/"]  # Catch-all last
   ```

2. **Test routes**:
   ```bash
   curl -v http://localhost:8000/
   curl -v http://localhost:8000/api/users
   curl -v http://localhost:8000/health
   ```

3. **Check route patterns**:
   ```yaml
   # Wrong: Missing leading /
   routes: ["api/*"]

   # Right
   routes: ["/api/*"]
   ```

### WebSocket Connection Failed

**Problem**: WebSocket upgrade fails.

**Solutions**:

1. **Check route includes WebSocket path**:
   ```yaml
   processes:
     frontend:
       routes: ["/", "/@vite/client"]  # Vite HMR
   ```

2. **Test WebSocket**:
   ```bash
   websocat ws://localhost:8000/ws
   # Or use browser DevTools Network tab
   ```

3. **Check backend WebSocket support**:
   ```javascript
   // Ensure backend handles WebSocket upgrade
   wss.on('connection', (ws) => {
     console.log('WebSocket connected');
   });
   ```

## Template Issues

### "Template not found"

**Problem**: Specified template doesn't exist.

**Solutions**:

1. **List available templates**:
   ```bash
   realm templates list
   ```

2. **Use correct name**:
   ```bash
   realm init --template=react-express
   # Not: realm init --template=react_express
   ```

3. **Create template first**:
   ```bash
   realm create --template=my-stack
   realm init --template=my-stack
   ```

### Template Files Not Copied

**Problem**: Expected files missing after init.

**Solutions**:

1. **Check template exists**:
   ```bash
   ls ~/.realm/templates/react-express/
   ```

2. **Recreate template**:
   ```bash
   cd original-project
   realm create --template=my-stack
   ```

3. **Manual copy**:
   ```bash
   cp -r ~/.realm/templates/my-stack/* ./
   ```

## Cache Issues

### Stale Version List

**Problem**: `realm list` shows old versions.

**Solution**: Clear cache:
```bash
realm cache clear
realm list --runtime=python
```

### Cache Permission Errors

**Problem**: Can't write to cache.

**Solution**: Fix permissions:
```bash
chmod -R u+w ~/.realm/cache/
```

## Deployment Issues

### Bundle Generation Fails

**Problem**: `realm bundle` errors.

**Solutions**:

1. **Check realm.yml valid**:
   ```bash
   realm start  # Test config first
   ```

2. **Check in activated environment**:
   ```bash
   source .venv/bin/activate
   realm bundle
   ```

3. **Check Docker installed** (if using deploy.sh):
   ```bash
   docker --version
   docker-compose --version
   ```

### Docker Build Fails

**Problem**: `./deploy.sh` fails.

**Solutions**:

1. **Check Dockerfile**:
   ```bash
   cd dist
   docker build -t test .
   # Read error messages
   ```

2. **Common issues**:
   - Missing dependencies in Dockerfile
   - Wrong COPY paths
   - Port mismatches

3. **Test docker-compose**:
   ```bash
   docker-compose config  # Validate syntax
   docker-compose up  # Without -d to see logs
   ```

## Performance Issues

### Slow Startup

**Problem**: `realm start` takes long.

**Solutions**:

1. **Check dependency installation**:
   ```bash
   # Pre-install dependencies
   cd frontend && npm install
   cd backend && npm install
   ```

2. **Use system runtime**:
   ```bash
   # Faster than downloading
   realm init --runtime=bun  # Uses system if available
   ```

3. **Reduce processes**:
   ```yaml
   # Start only what you need
   processes:
     frontend:
       command: "bun run dev"
   ```

### High Memory Usage

**Problem**: Realm using too much memory.

**Solutions**:

1. **Check process memory**:
   ```bash
   ps aux | grep bun
   ps aux | grep node
   ```

2. **Limit dev server memory**:
   ```yaml
   processes:
     frontend:
       command: "NODE_OPTIONS='--max-old-space-size=512' bun run dev"
   ```

3. **Stop unused processes**:
   ```bash
   realm stop
   ```

## Getting Help

### Enable Debug Logging

Add to realm.yml:
```yaml
env:
  DEBUG: "realm:*"
  LOG_LEVEL: debug
```

### Check Realm Version

```bash
realm --version
```

### Check System Info

```bash
uname -a  # OS info
which bun
which node
bun --version
node --version
```

### Report Issues

When reporting issues, include:

1. **Realm version**: `realm --version`
2. **OS**: `uname -a` or `systeminfo`
3. **Runtime**: `bun --version`, `node --version`, `python --version`
4. **realm.yml**: Configuration file
5. **Error messages**: Full output
6. **Steps to reproduce**: What you did

File issues at: https://github.com/wess/realm/issues

## Common Mistakes

### Not Activating Environment

```bash
# Wrong
realm start

# Right
source .venv/bin/activate
realm start
```

### Wrong Directory

```bash
# Wrong: Running from random directory
cd ~/Downloads
realm start  # No realm.yml here!

# Right: Run from project directory
cd ~/projects/myapp
realm start
```

### Missing Dependencies

```bash
# Wrong: Forgetting to install
realm init --template=react-express
source .venv/bin/activate
realm start  # Fails - no node_modules

# Right
realm init --template=react-express
source .venv/bin/activate
cd frontend && npm install
cd ../backend && npm install
realm start
```

### Hardcoded Ports

```javascript
// Wrong: Hardcoded backend URL
fetch('http://localhost:4001/api/users');

// Right: Use proxy
fetch('http://localhost:8000/api/users');

// Better: Use environment variable
fetch(`${import.meta.env.VITE_API_URL}/users`);
```

## Next Steps

- [Configuration](configuration.md) - Optimize your setup
- [Commands Reference](commands.md) - Learn all commands
- [Advanced Usage](advanced.md) - Complex workflows
