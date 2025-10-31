# Realm Runtime Recipes

Community-maintained runtime configurations for Realm.

## Available Runtimes

### Languages & Runtimes
- **[deno.yaml](deno.yaml)** - Deno runtime for JavaScript and TypeScript
- **[go.yaml](go.yaml)** - Go programming language
- **[zig.yaml](zig.yaml)** - Zig programming language and toolchain

### Tools
- **[ripgrep.yaml](ripgrep.yaml)** - Ripgrep (rg) - Fast line-oriented search tool

## Using a Runtime

1. **Copy to your config directory**:
   ```bash
   cp runtimes/deno.yaml ~/.realm/runtimes-config/
   ```

2. **Use with Realm**:
   ```bash
   realm init --runtime=deno
   realm init --runtime=deno@1.40.0
   ```

## Contributing a New Runtime

We welcome runtime contributions! Here's how to add a new runtime:

### 1. Create Your Manifest

Create `runtimes/yourruntime.yaml`:

```yaml
runtime:
  name: yourruntime
  display_name: Your Runtime
  aliases:
    - alias1
  version_command: "--version"
  description: Brief description
  versions_url: "https://github.com/owner/repo/releases"

versions:
  type: github  # or "api" or "static"
  repo: owner/repo
  tag_pattern: "^v(.+)$"

downloads:
  darwin-arm64:
    url_template: "https://example.com/{version}/binary-macos-arm64.tar.gz"
    format: tar.gz
    checksum_url: "https://example.com/{version}/checksums.txt"
    checksum_algo: sha256

  darwin-x64:
    url_template: "https://example.com/{version}/binary-macos-x64.tar.gz"
    format: tar.gz

  linux-x64:
    url_template: "https://example.com/{version}/binary-linux-x64.tar.gz"
    format: tar.gz

  linux-arm64:
    url_template: "https://example.com/{version}/binary-linux-arm64.tar.gz"
    format: tar.gz

install:
  binary_path: bin/yourruntime
  additional_binaries:
    - tool1
    - tool2
  strip_components: 1
  post_install_commands: []

environment:
  requires_isolation: false
  vars: {}
```

### 2. Test Your Manifest

```bash
# Copy to config
cp runtimes/yourruntime.yaml ~/.realm/runtimes-config/

# Test version listing
realm list --runtime=yourruntime

# Test installation
mkdir test-runtime && cd test-runtime
realm init --runtime=yourruntime
source .venv/bin/activate
yourruntime --version
```

### 3. Submit a Pull Request

1. Fork the repository
2. Create your runtime manifest in `runtimes/`
3. Test thoroughly on your platform
4. Add your runtime to the list in this README
5. Submit a PR with:
   - Runtime manifest (`runtimes/yourruntime.yaml`)
   - Updated README (add to Available Runtimes list)
   - Brief description of the runtime
   - Platforms tested

### Manifest Requirements

✅ **Required**:
- Support at least 2 platforms (darwin-arm64, darwin-x64, linux-x64, linux-arm64)
- Working `versions_url` that returns actual versions
- Valid download URLs (HTTPS only)
- Tested on at least one platform

✅ **Recommended**:
- Checksums for downloads
- Support for all 4 major platforms
- Clear description
- Latest version tested

❌ **Not Allowed**:
- HTTP URLs (must be HTTPS)
- Untrusted download sources
- Malware or malicious code
- Copyright violations

## Runtime Guidelines

### Supported Version Discovery

**GitHub Releases** (recommended):
```yaml
versions:
  type: github
  repo: owner/repo
  tag_pattern: "^v(.+)$"  # Optional: extract version from tag
```

**JSON API**:
```yaml
versions:
  type: api
  url: "https://api.example.com/versions"
  json_path: data.versions
```

**Static List** (for testing):
```yaml
versions:
  type: static
  versions:
    - "1.0.0"
    - "1.1.0"
```

### Platform Naming

Use these exact platform keys:
- `darwin-arm64` - macOS Apple Silicon
- `darwin-x64` - macOS Intel
- `linux-x64` - Linux x86_64
- `linux-arm64` - Linux ARM64
- `windows-x64` - Windows (future support)

### URL Templates

Template variables:
- `{version}` - Version being installed
- `{os}` - Operating system (darwin, linux, windows)
- `{arch}` - Architecture (x64, arm64)

Example:
```yaml
url_template: "https://releases.example.com/{version}/binary-{os}-{arch}.tar.gz"
```

### Archive Formats

Supported formats:
- `tar.gz` - Gzipped tar archive (most common)
- `zip` - Zip archive
- `binary` - Raw binary (no extraction)

### Binary Paths

Path to the main executable in the extracted archive:

```yaml
install:
  binary_path: bin/myruntime  # Relative to archive root
  strip_components: 0          # Number of path components to strip
```

If archive structure is:
```
myruntime-1.0.0/
  bin/
    myruntime
```

Then:
```yaml
binary_path: myruntime-1.0.0/bin/myruntime
strip_components: 0

# OR

binary_path: bin/myruntime
strip_components: 1
```

## Testing Checklist

Before submitting, verify:

- [ ] Manifest parses without errors
- [ ] `realm list --runtime=yourruntime` shows versions
- [ ] `realm init --runtime=yourruntime` installs successfully
- [ ] Binary is executable and runs
- [ ] `yourruntime --version` works
- [ ] Works on your platform (darwin-arm64, darwin-x64, linux-x64, or linux-arm64)
- [ ] Download URLs are HTTPS
- [ ] Checksums verify (if provided)
- [ ] No sensitive data in manifest

## Popular Runtimes to Add

Looking for contribution ideas? Consider adding:

### Languages
- **Ruby** - Ruby programming language
- **PHP** - PHP interpreter
- **Rust** - Rust toolchain
- **Zig** - Zig compiler
- **Swift** - Swift compiler
- **Kotlin** - Kotlin compiler

### JavaScript Runtimes
- **Deno** ✅ (available)
- **Node.js** (built-in, but can add specific builds)
- **Bun** (built-in, but can add specific builds)

### System Tools
- **Just** - Command runner
- **Watchexec** - File watcher
- **Ripgrep** - Fast grep
- **Fd** - Fast find
- **Bat** - Cat clone with syntax highlighting

### Build Tools
- **Make** - GNU Make
- **CMake** - Cross-platform build system
- **Meson** - Build system
- **Bazel** - Build system

## Examples

See existing runtimes for reference:
- [deno.yaml](deno.yaml) - Simple single-binary runtime
- [go.yaml](go.yaml) - Archive-based with additional binaries

## Getting Help

- [Custom Runtimes Documentation](../docs/custom-runtimes.md)
- [Runtime Plugin Architecture](../RUNTIME_PLUGIN_ARCHITECTURE.md)
- [Open an Issue](https://github.com/wess/realm/issues)

## License

Runtime manifests are provided under MIT license.
Each runtime binary is subject to its own license.
