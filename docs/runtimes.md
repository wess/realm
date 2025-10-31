# Runtime Management

Realm automatically manages Bun, Node.js, and Python installations for your projects.

## Supported Runtimes

- **Bun** - Fast JavaScript runtime with built-in bundler
- **Node.js** - JavaScript runtime
- **Python** - Python programming language

## Specifying Runtimes

### Format

```
runtime[@version]
```

### Examples

```bash
# Latest version
realm init --runtime=bun
realm init --runtime=node
realm init --runtime=python

# Major version (latest patch)
realm init --runtime=node@20
realm init --runtime=python@3.12

# Specific version
realm init --runtime=bun@1.1.34
realm init --runtime=node@20.18.0
realm init --runtime=python@3.12.6
```

## Version Resolution

### Latest

```bash
realm init --runtime=bun
```

Realm fetches the latest stable version:
- Checks system PATH for existing installation
- If found and recent, uses system version
- Otherwise, downloads latest to `~/.realm/`

### Major Version

```bash
realm init --runtime=node@20
```

Resolves to latest 20.x release:
- `node@20` → `node@20.18.0` (example)
- Always downloads to `~/.realm/`

### Specific Version

```bash
realm init --runtime=python@3.12.6
```

Installs exactly 3.12.6:
- Always downloads to `~/.realm/`
- Ensures reproducibility

## Listing Available Versions

### Python

```bash
realm list --runtime=python
```

Output:
```
📦 Fetching available Python versions...
   (using cached data)

   Available versions:
   ✓ 3.12.6 (installed)
   • 3.12.5
   • 3.12.4
   • 3.11.9
   • 3.11.8
   • 3.11.7
   ...
```

### Node.js

```bash
realm list --runtime=node
```

Shows all available Node.js versions with LTS indicators.

### Bun

```bash
realm list --runtime=bun
```

Shows all available Bun releases.

## Runtime Storage

### Directory Structure

```
~/.realm/
├── runtimes/
│   ├── bun-1.1.34/
│   │   └── bun
│   ├── node-20.18.0/
│   │   ├── bin/
│   │   │   ├── node
│   │   │   └── npm
│   │   └── lib/
│   └── python-3.12.6/
│       ├── bin/
│       │   └── python
│       └── lib/
└── cache/
    └── versions.json
```

### Sharing Between Projects

Multiple projects can use the same runtime:

```bash
# Project 1
realm init project1 --runtime=bun@1.1.34
# Downloads bun-1.1.34 to ~/.realm/runtimes/

# Project 2
realm init project2 --runtime=bun@1.1.34
# Reuses existing bun-1.1.34, no download
```

## System vs Downloaded Runtimes

### System Runtime

When using `--runtime=bun` (latest), Realm checks if bun exists in PATH:

```bash
realm init --runtime=bun
# If bun is in PATH → uses system bun
# Output: ✓ Using system-installed bun (found in PATH)
```

**Benefits**:
- Faster initialization (no download)
- Uses system-managed version
- Saves disk space

### Downloaded Runtime

When using specific versions, Realm always downloads:

```bash
realm init --runtime=bun@1.1.34
# Always downloads to ~/.realm/runtimes/bun-1.1.34/
# Output: 📦 Getting bun 1.1.34...
```

**Benefits**:
- Exact version control
- Isolated from system changes
- Reproducible across machines

## Runtime Details

### Bun

**Source**: [bun.sh releases](https://github.com/oven-sh/bun/releases)

**Platforms**:
- macOS (x86_64, ARM64)
- Linux (x86_64, ARM64)
- Windows (x86_64)

**Features**:
- Fast JavaScript runtime
- Built-in package manager
- TypeScript support out-of-the-box
- Compatible with Node.js APIs

**Usage**:
```bash
realm init --runtime=bun
source .venv/bin/activate
bun --version
```

### Node.js

**Source**: [nodejs.org releases](https://nodejs.org/dist/)

**Platforms**:
- macOS (x86_64, ARM64)
- Linux (x86_64, ARM64)
- Windows (x86_64)

**Features**:
- Mature JavaScript runtime
- Huge ecosystem (npm)
- Long-term support (LTS) versions
- Industry standard

**Usage**:
```bash
realm init --runtime=node@20
source .venv/bin/activate
node --version
npm --version
```

**LTS Versions**:
- Node.js 20 - LTS until April 2026
- Node.js 18 - LTS until April 2025
- Node.js 16 - End of life

### Python

**Source**: [python-build-standalone](https://github.com/indygreg/python-build-standalone)

**Platforms**:
- macOS (x86_64, ARM64)
- Linux (x86_64)
- Windows (x86_64)

**Features**:
- Fully isolated Python environment
- Per-project site-packages
- Compatible with pip, poetry, etc.
- No system Python pollution

**Usage**:
```bash
realm init --runtime=python@3.12
source .venv/bin/activate
python --version
pip install -r requirements.txt
```

**Python Isolation**:

Realm creates true isolation for Python:
```
.venv/
├── bin/
│   └── python -> ~/.realm/runtimes/python-3.12.6/bin/python
├── lib/
│   └── site-packages/  # Project-local packages
└── pyvenv.cfg
```

Environment variables set:
- `VIRTUAL_ENV` - Points to .venv
- `PYTHONHOME` - Not set (isolation)
- `PATH` - Includes .venv/bin

## Downloading Runtimes

### Download Process

When Realm downloads a runtime:

1. **Resolve version**:
   ```
   📦 Getting bun 1.1.34...
   ```

2. **Download with progress**:
   ```
   ⠋ [████████████████████░░] 15.2 MB / 18.5 MB (bun)
   ```

3. **Extract**:
   ```
   📦 Extracting...
   ```

4. **Verify**:
   ```
   ✓ bun 1.1.34 installed
   ```

### Download Sources

- **Bun**: `https://github.com/oven-sh/bun/releases`
- **Node.js**: `https://nodejs.org/dist/`
- **Python**: `https://github.com/indygreg/python-build-standalone/releases`

### Network Requirements

- HTTPS access to download sources
- ~20-100 MB download per runtime
- TLS 1.2+ support

## Version Caching

### Cache Behavior

Version lists are cached for 24 hours:

```bash
realm list --runtime=python
# First run: fetches from network
# Output: 📦 Fetching available Python versions...

realm list --runtime=python
# Within 24h: uses cache
# Output: 📦 Fetching available Python versions...
#         (using cached data)
```

### Clear Cache

```bash
realm cache clear
```

Forces fresh fetch on next `realm list` command.

### Cache Location

```
~/.realm/
└── cache/
    ├── bun-versions.json
    ├── node-versions.json
    └── python-versions.json
```

## Per-Project Runtimes

Each project gets its own runtime reference:

```bash
# Project 1: Bun
cd project1
realm init --runtime=bun
source .venv/bin/activate
which bun  # → /path/to/project1/.venv/bin/bun

# Project 2: Node.js
cd project2
realm init --runtime=node@20
source .venv/bin/activate
which node  # → /path/to/project2/.venv/bin/node

# No conflicts!
```

## Upgrading Runtimes

### Create New Environment

The safest approach:

```bash
# Current environment
source .venv/bin/activate
bun --version  # 1.1.34

# Deactivate
deactivate

# Backup old environment
mv .venv .venv.bak

# Create new environment with updated runtime
realm init --runtime=bun@1.1.40

# Test new environment
source .venv/bin/activate
bun --version  # 1.1.40

# If all works, remove backup
rm -rf .venv.bak
```

### Why Not In-Place Upgrade?

Runtime upgrades can break dependencies:
- npm packages may be incompatible
- Python packages may need recompilation
- Path configurations may change

Creating a new environment ensures clean state.

## Multi-Runtime Projects

Use different runtimes in the same workflow:

```bash
# Backend with Python
realm init backend --runtime=python@3.12
cd backend
source .venv/bin/activate
pip install fastapi uvicorn
deactivate

# Frontend with Bun
realm init frontend --runtime=bun
cd frontend
source .venv/bin/activate
bun add react react-dom
deactivate

# Full-stack with shared realm.yml
# Both environments exist separately
```

## Runtime Compatibility

### Package Managers

**Bun Projects**:
```bash
bun install        # Install dependencies
bun add package    # Add package
bun run script     # Run script
```

**Node.js Projects**:
```bash
npm install        # Install dependencies
npm install package # Add package
npm run script     # Run script
```

**Python Projects**:
```bash
pip install -r requirements.txt  # Install dependencies
pip install package              # Add package
python script.py                 # Run script
```

### Switching Between Runtimes

A project created with Bun can work with Node.js:

```bash
# Originally created with Bun
realm init --runtime=bun --template=react-express

# Can use Node.js instead
realm init --runtime=node@20 --template=react-express

# realm.yml commands work with both
```

## Platform Support

### macOS

All runtimes supported:
- Intel (x86_64)
- Apple Silicon (ARM64)

### Linux

All runtimes supported:
- x86_64
- ARM64 (aarch64)

### Windows

All runtimes supported:
- x86_64

Platform detected automatically.

## Troubleshooting

### "Runtime not found"

```bash
realm init --runtime=bun@99.99.99
# Error: Version 99.99.99 not found
```

**Solution**: Check available versions:
```bash
realm list --runtime=bun
```

### "Download failed"

```bash
# Error: Failed to download runtime
```

**Solutions**:
- Check internet connection
- Verify firewall allows HTTPS
- Try again (transient network issue)
- Clear cache: `realm cache clear`

### "Permission denied"

```bash
# Error: Permission denied writing to ~/.realm/
```

**Solution**: Check directory permissions:
```bash
ls -la ~/.realm
chmod -R u+w ~/.realm
```

### "Incompatible platform"

```bash
# Error: No binary available for platform
```

**Solution**: Your OS/architecture isn't supported for that runtime version. Try:
- A different version
- A different runtime (Bun vs Node.js)
- System-installed runtime

## Best Practices

### Use Specific Versions in Production

```bash
# Development: use latest
realm init --runtime=bun

# Production: pin version
realm init --runtime=bun@1.1.34
```

### Document Runtime Requirements

In README.md:
```markdown
## Requirements

- Bun 1.1.34+
- Or Node.js 20+
```

### Test Multiple Versions

```bash
# Test with Node 18
realm init test18 --runtime=node@18
cd test18 && source .venv/bin/activate
npm test

# Test with Node 20
realm init test20 --runtime=node@20
cd test20 && source .venv/bin/activate
npm test
```

### Keep Runtimes Updated

Periodically upgrade:

```bash
# Check current
bun --version  # 1.1.34

# Check available
realm list --runtime=bun

# Upgrade if needed
deactivate
mv .venv .venv.old
realm init --runtime=bun@1.1.40
source .venv/bin/activate
```

## Next Steps

- [Process Management](processes.md) - Configure services
- [Templates](templates.md) - Use templates with different runtimes
- [Deployment](deployment.md) - Deploy with specific runtimes
