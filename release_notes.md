# Realm v0.1.0 - Initial Release

## 🎉 Features

### Core Functionality
- **Isolated Development Environments**: Virtualenv-like project isolation with per-project runtime versions
- **Multi-Runtime Support**: Automatic management of Bun and Node.js versions
- **Process Management**: Foreman-like parallel process orchestration with lifecycle management
- **Intelligent HTTP Proxy**: Zero-config routing with WebSocket support and CORS handling
- **Project Templates**: Built-in templates for React, Svelte, Vue, and Next.js stacks
- **Deployment Generation**: One-command Docker and nginx configuration creation

### Runtime Management
- Download and install specific Bun/Node.js versions per project
- Automatic PATH configuration via activation scripts
- No global runtime pollution - complete isolation

### Development Workflow
- Simple `realm init` to create new environments
- Shell activation similar to Python virtualenv
- Combined process and proxy management with `realm start`
- Clean configuration via `realm.yml`

### Built-in Templates
- react-express: React frontend with Express backend
- svelte-fastify: Svelte frontend with Fastify backend  
- vue-express: Vue.js frontend with Express backend
- nextjs: Next.js full-stack application

## 📦 Installation

### From crates.io
```bash
cargo install realm
```

### Pre-built Binaries
Available for:
- macOS (Intel & Apple Silicon)
- Linux (x64 & ARM64)
- Windows (x64)

### Quick Install (macOS/Linux)
```bash
curl -sSfL https://github.com/wess/realm/releases/latest/download/install.sh | bash
```

## 🚀 Quick Start

```bash
# Create a new project
realm init myapp --template=react-express --runtime=bun

# Activate environment
cd myapp
source .venv/bin/activate

# Start development
realm start
```

## 📋 Requirements

- Rust 1.75+ (for building from source)
- No runtime dependencies - Realm manages everything!

## 🔮 Coming Next

- v0.2.0: Kubernetes/Helm deployment support
- v0.3.0: IDE extensions and live configuration reload
- v0.4.0: Python, Go, Rust, and Deno runtime support

## 📝 License

MIT License - See LICENSE file for details

## 🙏 Acknowledgments

Initial release by @wess and the Realm contributors.

---

**Full Changelog**: https://github.com/wess/realm/commits/v0.1.0
