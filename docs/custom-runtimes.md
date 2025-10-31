# Custom Runtimes

Realm supports a plugin system for adding custom runtimes without modifying core code.

## Overview

Runtimes can be added in two ways:

1. **Declarative** - Define runtime via YAML manifest (easiest)
2. **Programmatic** - Implement `RuntimeProvider` trait in Rust (advanced)

## Declarative Runtimes

### Quick Start

#### Option 1: Use Community Runtimes

Browse available runtimes in the [runtimes/](https://github.com/wess/realm/tree/main/runtimes) folder:

```bash
# Copy a runtime recipe
curl -o ~/.realm/runtimes-config/deno.yaml \
  https://raw.githubusercontent.com/wess/realm/main/runtimes/deno.yaml

# Use it
realm init --runtime=deno
```

#### Option 2: Create Your Own

1. Create a YAML file in `~/.realm/runtimes-config/`:
   ```bash
   mkdir -p ~/.realm/runtimes-config
   ```

2. Define your runtime (e.g., `deno.yaml`):
   ```yaml
   runtime:
     name: deno
     display_name: Deno
     versions_url: "https://github.com/denoland/deno/releases"

   versions:
     type: github
     repo: denoland/deno

   downloads:
     darwin-arm64:
       url_template: "https://github.com/denoland/deno/releases/download/v{version}/deno-aarch64-apple-darwin.zip"
       format: zip

   install:
     binary_path: deno
   ```

3. Use it:
   ```bash
   realm init --runtime=deno
   ```

### Manifest Structure

#### Runtime Metadata

```yaml
runtime:
  name: deno              # Required: Unique identifier
  display_name: Deno      # Optional: Display name
  aliases:                # Optional: Alternative names
    - deno
  version_command: "--version"  # Optional: Command to check version (default: --version)
  description: A secure runtime for JavaScript and TypeScript  # Optional
  versions_url: "https://github.com/denoland/deno/releases"  # Required: Where to find versions
```

The `versions_url` is the primary source for discovering available versions. It can point to:
- GitHub releases page
- JSON API endpoint
- HTML page with version list

#### Version Discovery

Realm needs to know where to find available versions.

**GitHub Releases**:
```yaml
versions:
  type: github
  repo: denoland/deno
  tag_pattern: "^v(.+)$"    # Optional: Extract version from tag
  filter: stable            # Optional: Filter releases
```

**JSON API**:
```yaml
versions:
  type: api
  url: "https://api.example.com/versions"
  json_path: data.versions  # JSONPath to version array
```

**Static List** (for testing):
```yaml
versions:
  type: static
  versions:
    - "1.0.0"
    - "1.1.0"
    - "1.2.0"
```

#### Download Configuration

Define downloads per platform. Format: `downloads.{os}-{arch}`

Supported platforms:
- `darwin-x64` - macOS Intel
- `darwin-arm64` - macOS Apple Silicon
- `linux-x64` - Linux x86_64
- `linux-arm64` - Linux ARM64
- `windows-x64` - Windows x64

```yaml
downloads:
  darwin-arm64:
    url_template: "https://example.com/releases/{version}/binary-{os}-{arch}.tar.gz"
    format: tar.gz          # Options: "tar.gz", "zip", "binary"

    # Optional: Checksums
    checksum_url: "https://example.com/releases/{version}/checksums.txt"
    checksum_algo: sha256   # Options: "sha256", "sha512"

    # Optional: OS/arch name mapping
    os_map:
      darwin: macos           # Maps realm's "darwin" to runtime's "macos"

    arch_map:
      arm64: aarch64          # Maps realm's "arm64" to runtime's "aarch64"
```

**URL Template Variables**:
- `{version}` - Version being installed
- `{os}` - Operating system (darwin, linux, windows)
- `{arch}` - Architecture (x64, arm64)

#### Installation Configuration

```yaml
install:
  binary_path: bin/deno           # Path to main executable in archive
  additional_binaries:            # Optional: Additional binaries to symlink
    - gofmt
  strip_components: 1             # Optional: Strip N leading path components
  post_install_commands:          # Optional: Commands to run after extraction
    - "chmod +x bin/*"
    - "./install.sh"
```

#### Environment Configuration

```yaml
environment:
  requires_isolation: false       # Whether this runtime needs virtualenv-like isolation

  # Optional: Environment variables to set
  vars:
    DENO_DIR: "{install_dir}/.deno"
    PATH: "{install_dir}/bin:$PATH"

  # Optional: Commands for isolation setup (if requires_isolation = true)
  isolation_commands:
    - "python -m venv {venv_dir}"
```

## Complete Examples

### Deno Runtime

```yaml
# ~/.realm/runtimes-config/deno.yaml

runtime:
  name: deno
  display_name: Deno
  aliases: []
  description: A secure runtime for JavaScript and TypeScript
  versions_url: "https://github.com/denoland/deno/releases"

versions:
  type: github
  repo: denoland/deno
  tag_pattern: "^v(.+)$"

downloads:
  darwin-arm64:
    url_template: "https://github.com/denoland/deno/releases/download/v{version}/deno-aarch64-apple-darwin.zip"
    format: zip

  darwin-x64:
    url_template: "https://github.com/denoland/deno/releases/download/v{version}/deno-x86_64-apple-darwin.zip"
    format: zip

  linux-x64:
    url_template: "https://github.com/denoland/deno/releases/download/v{version}/deno-x86_64-unknown-linux-gnu.zip"
    format: zip

install:
  binary_path: deno
  strip_components: 0

environment:
  requires_isolation: false
```

Usage:
```bash
realm init --runtime=deno
realm init --runtime=deno@1.40.0
```

### Go Runtime

```yaml
# ~/.realm/runtimes-config/go.yaml

runtime:
  name: go
  display_name: Go
  aliases:
    - golang
  version_command: version
  versions_url: "https://go.dev/dl/?mode=json"

versions:
  type: api
  url: "https://go.dev/dl/?mode=json"
  json_path: version

downloads:
  darwin-arm64:
    url_template: "https://go.dev/dl/go{version}.darwin-arm64.tar.gz"
    format: tar.gz
    checksum_url: "https://go.dev/dl/go{version}.darwin-arm64.tar.gz.sha256"
    checksum_algo: sha256

  linux-x64:
    url_template: "https://go.dev/dl/go{version}.linux-amd64.tar.gz"
    format: tar.gz

install:
  binary_path: go/bin/go
  additional_binaries:
    - gofmt
  strip_components: 0

environment:
  requires_isolation: false
  vars:
    GOROOT: "{install_dir}/go"
```

Usage:
```bash
realm init --runtime=go
realm init --runtime=golang@1.21.0
```

### Ruby Runtime

```yaml
# ~/.realm/runtimes-config/ruby.yaml

runtime:
  name: ruby
  display_name: Ruby
  aliases:
    - rb
  versions_url: "https://github.com/ruby/ruby/releases"

versions:
  type: github
  repo: ruby/ruby
  tag_pattern: "^v(.+)$"

downloads:
  darwin-arm64:
    url_template: "https://cache.ruby-lang.org/pub/ruby/{version}/ruby-{version}.tar.gz"
    format: tar.gz

  linux-x64:
    url_template: "https://cache.ruby-lang.org/pub/ruby/{version}/ruby-{version}.tar.gz"
    format: tar.gz

install:
  binary_path: bin/ruby
  additional_binaries:
    - gem
    - irb
    - rake
  strip_components: 1
  post_install_commands:
    - "./configure --prefix={install_dir}"
    - make
    - make install

environment:
  requires_isolation: false
```

### Zig Runtime

```yaml
# ~/.realm/runtimes-config/zig.yaml

runtime:
  name: zig
  display_name: Zig
  versions_url: "https://github.com/ziglang/zig/releases"

versions:
  type: github
  repo: ziglang/zig

downloads:
  darwin-arm64:
    url_template: "https://ziglang.org/download/{version}/zig-macos-aarch64-{version}.tar.xz"
    format: tar.gz

  linux-x64:
    url_template: "https://ziglang.org/download/{version}/zig-linux-x86_64-{version}.tar.xz"
    format: tar.gz

install:
  binary_path: zig
  strip_components: 1

environment:
  requires_isolation: false
```

## Programmatic Runtimes (Advanced)

For complex runtime requirements, implement the `RuntimeProvider` trait:

```rust
use async_trait::async_trait;
use realm::runtime::provider::{RuntimeProvider, PlatformInfo, RuntimeArtifact};

pub struct CustomRuntime;

#[async_trait]
impl RuntimeProvider for CustomRuntime {
    fn name(&self) -> &str {
        "custom"
    }

    async fn list_versions(&self) -> Result<Vec<String>> {
        // Fetch available versions
        Ok(vec!["1.0.0".to_string()])
    }

    async fn get_artifact(&self, version: &str, platform: &PlatformInfo) -> Result<RuntimeArtifact> {
        // Return download URL and metadata
        Ok(RuntimeArtifact {
            url: format!("https://example.com/{version}/binary.tar.gz"),
            checksum: None,
            checksum_algo: None,
            format: ArtifactFormat::TarGz,
        })
    }

    async fn install_artifact(
        &self,
        artifact_data: &[u8],
        artifact: &RuntimeArtifact,
        install_dir: &PathBuf,
    ) -> Result<()> {
        // Extract and install
        Ok(())
    }
}
```

Register in `RuntimeRegistry`:
```rust
let mut registry = RuntimeRegistry::new();
registry.register(Arc::new(CustomRuntime));
```

## Testing Your Runtime

### 1. Create Manifest

```bash
cat > ~/.realm/runtimes-config/myruntime.yaml << 'EOF'
runtime:
  name: myruntime
  display_name: My Runtime
  versions_url: "https://example.com/versions"

versions:
  type: static
  versions:
    - "1.0.0"

downloads:
  darwin-arm64:
    url_template: "https://example.com/myruntime-{version}.zip"
    format: zip

install:
  binary_path: myruntime
EOF
```

### 2. Test Discovery

```bash
# Check if runtime is detected
realm list --runtime=myruntime
```

### 3. Test Installation

```bash
realm init test --runtime=myruntime@1.0.0
cd test
source .venv/bin/activate
which myruntime
```

### 4. Debug

Enable debug logging:
```bash
export RUST_LOG=debug
realm init --runtime=myruntime
```

## Best Practices

### URL Templates

Use official release URLs:
```yaml
# Good
downloads:
  darwin-arm64:
    url_template: "https://github.com/org/repo/releases/download/v{version}/binary.zip"

# Avoid
downloads:
  darwin-arm64:
    url_template: "https://random-mirror.com/binary.zip"
```

### Checksums

Always verify downloads when possible:
```yaml
downloads:
  darwin-arm64:
    checksum_url: "https://releases.example.com/{version}/SHA256SUMS"
    checksum_algo: sha256
```

### Version Discovery

Use reliable sources:
```yaml
# GitHub releases (most reliable)
versions:
  type: github
  repo: owner/repo

# Official API
versions:
  type: api
  url: "https://official-api.example.com/versions"
```

### Platform Support

Support major platforms:
```yaml
downloads:
  darwin-arm64:  # macOS Apple Silicon
  darwin-x64:     # macOS Intel
  linux-x64:      # Linux x86_64
  linux-arm64:    # Linux ARM64
```

### Binary Paths

Use correct paths after extraction:
```yaml
# If archive contains:
#   myruntime-1.0.0/
#     bin/
#       myruntime

install:
  binary_path: myruntime-1.0.0/bin/myruntime
  strip_components: 0

# Or strip leading component:
install:
  binary_path: bin/myruntime
  strip_components: 1
```

## Sharing Runtimes

### Package as Template

```bash
# Create shareable template
mkdir -p realm-runtime-deno
cp ~/.realm/runtimes-config/deno.yaml realm-runtime-deno/
cat > realm-runtime-deno/README.md << 'EOF'
# Deno Runtime for Realm

## Installation

```bash
cp deno.yaml ~/.realm/runtimes-config/
```

## Usage

```bash
realm init --runtime=deno
```
EOF

# Share
tar -czf realm-runtime-deno.tar.gz realm-runtime-deno/
```

### Publish to GitHub

```bash
git init
git add .
git commit -m "Add Deno runtime for Realm"
gh repo create realm-runtime-deno --public
git push origin main
```

Users install:
```bash
git clone https://github.com/you/realm-runtime-deno
cp realm-runtime-deno/deno.yaml ~/.realm/runtimes-config/
```

## Troubleshooting

### Runtime Not Detected

```bash
# Check file location
ls ~/.realm/runtimes-config/

# Check YAML syntax
cat ~/.realm/runtimes-config/myruntime.yaml

# Check realm finds it
export RUST_LOG=debug
realm init --runtime=myruntime
```

### Version Fetch Failed

```bash
# Test version URL manually
curl https://api.github.com/repos/owner/repo/releases

# Check JSON structure matches json_path
```

### Download Failed

```bash
# Test download URL
curl -L "https://example.com/runtime-1.0.0.zip"

# Check platform key is correct
# Must be: darwin-arm64, darwin-x64, linux-x64, linux-arm64
```

### Installation Failed

```bash
# Check archive structure
tar -tzf runtime-1.0.0.tar.gz | head

# Ensure binary_path matches
# Update strip_components if needed
```

## Migration from Built-in Runtimes

Existing runtimes (Bun, Node, Python) will eventually migrate to the plugin system. You can already create custom versions:

```yaml
# ~/.realm/runtimes-config/node-custom.yaml
runtime:
  name: node-custom
  display_name: Node.js (Custom)
  versions_url: "https://my-custom-builds.com/versions"

# Custom Node.js build or fork
downloads:
  darwin-arm64:
    url_template: "https://my-custom-builds.com/node-{version}.tar.gz"
    format: tar.gz

install:
  binary_path: bin/node
  additional_binaries:
    - npm
    - npx
```

Usage:
```bash
realm init --runtime=node-custom@20.0.0
```

## Community Runtimes

### Browse Available Runtimes

Check the [runtimes/](https://github.com/wess/realm/tree/main/runtimes) folder for community-maintained recipes:
- Deno, Go, Zig, Just, and more
- Tested and verified configurations
- Ready to use

### Contribute a Runtime

Have a runtime recipe to share? Submit a PR!

1. **Create manifest**: `runtimes/yourruntime.yaml`
2. **Test thoroughly**: Verify on your platform
3. **Submit PR**: Add to [realm/runtimes/](https://github.com/wess/realm/tree/main/runtimes)

See [runtimes/README.md](https://github.com/wess/realm/blob/main/runtimes/README.md) for contribution guidelines.

## Next Steps

- [Runtime Management](runtimes.md) - Understand built-in runtimes
- [Configuration](configuration.md) - Configure realm.yml
- [Advanced Usage](advanced.md) - Complex patterns
