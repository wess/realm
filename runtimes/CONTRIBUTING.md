# Contributing Runtime Recipes

Thank you for contributing to Realm's runtime collection!

## Before You Start

1. **Check if it exists**: Search [existing runtimes](README.md#available-runtimes)
2. **Official source**: Ensure downloads come from official/trusted sources
3. **Multi-platform**: Support at least 2 platforms
4. **Test locally**: Verify it works on your platform

## Step-by-Step Guide

### 1. Create the Manifest

Create `runtimes/yourruntime.yaml`:

```yaml
runtime:
  name: yourruntime              # Unique identifier (lowercase, no spaces)
  display_name: Your Runtime     # Human-readable name
  aliases:                       # Optional alternative names
    - yr
  version_command: "--version"   # Command to check version
  description: Brief description of what this runtime does
  versions_url: "https://github.com/owner/repo/releases"  # Where versions are listed

versions:
  type: github                   # "github", "api", or "static"
  repo: owner/repo               # For GitHub releases
  tag_pattern: "^v(.+)$"         # Optional: Extract version from tag

downloads:
  darwin-arm64:                  # macOS Apple Silicon
    url_template: "https://releases.example.com/{version}/binary-darwin-arm64.tar.gz"
    format: tar.gz               # "tar.gz", "zip", or "binary"
    checksum_url: "https://releases.example.com/{version}/checksums.txt"  # Optional
    checksum_algo: sha256        # Optional: "sha256" or "sha512"

  darwin-x64:                    # macOS Intel
    url_template: "https://releases.example.com/{version}/binary-darwin-x64.tar.gz"
    format: tar.gz

  linux-x64:                     # Linux x86_64
    url_template: "https://releases.example.com/{version}/binary-linux-x64.tar.gz"
    format: tar.gz

  linux-arm64:                   # Linux ARM64
    url_template: "https://releases.example.com/{version}/binary-linux-arm64.tar.gz"
    format: tar.gz

install:
  binary_path: bin/yourruntime   # Path to executable in archive
  additional_binaries:           # Optional: Other binaries to symlink
    - tool1
    - tool2
  strip_components: 1            # Number of path components to strip
  post_install_commands: []      # Optional: Commands to run after extraction

environment:
  requires_isolation: false      # Set to true for Python-like isolation
  vars: {}                       # Optional: Environment variables to set
```

### 2. Test Your Manifest

```bash
# Copy to config directory
cp runtimes/yourruntime.yaml ~/.realm/runtimes-config/

# Test version listing
realm list --runtime=yourruntime

# Test installation
mkdir /tmp/test-runtime && cd /tmp/test-runtime
realm init --runtime=yourruntime
source .venv/bin/activate

# Verify it works
yourruntime --version

# Test with specific version
cd /tmp
rm -rf test-runtime
mkdir test-runtime && cd test-runtime
realm init --runtime=yourruntime@1.0.0
source .venv/bin/activate
yourruntime --version

# Clean up
cd /tmp && rm -rf test-runtime
```

### 3. Update README

Add your runtime to [README.md](README.md):

```markdown
### Category
- **[yourruntime.yaml](yourruntime.yaml)** - Brief description
```

Categories:
- **Languages & Runtimes** - Programming languages and their runtimes
- **Tools** - CLI tools, build systems, etc.

### 4. Create Pull Request

1. Fork the repository
2. Create a branch: `git checkout -b add-yourruntime`
3. Add your files:
   ```bash
   git add runtimes/yourruntime.yaml
   git add runtimes/README.md
   ```
4. Commit: `git commit -m "Add yourruntime runtime"`
5. Push: `git push origin add-yourruntime`
6. Open a PR at https://github.com/wess/realm/pulls

### 5. PR Description Template

```markdown
## Add [Runtime Name] Runtime

### Description
Brief description of what this runtime does.

### Testing
- [ ] Tested on macOS (ARM64/Intel)
- [ ] Tested on Linux (x64/ARM64)
- [ ] Version listing works (`realm list --runtime=yourruntime`)
- [ ] Installation works (`realm init --runtime=yourruntime`)
- [ ] Binary executes (`yourruntime --version`)
- [ ] Specific version works (`realm init --runtime=yourruntime@1.0.0`)

### Platform Support
- [x] darwin-arm64
- [x] darwin-x64
- [x] linux-x64
- [ ] linux-arm64

### Links
- Homepage: https://example.com
- Releases: https://github.com/owner/repo/releases
- Documentation: https://docs.example.com
```

## Checklist

Before submitting your PR, verify:

- [ ] Manifest parses without errors
- [ ] `runtime.name` is unique and lowercase
- [ ] `versions_url` points to actual releases
- [ ] Download URLs are HTTPS only
- [ ] Supports at least 2 platforms
- [ ] Tested on at least 1 platform
- [ ] Binary path is correct
- [ ] `strip_components` is set correctly
- [ ] Added to README.md
- [ ] No sensitive data in manifest
- [ ] Follows existing naming conventions

## Common Issues

### Archive Structure

Understand your archive structure:

```bash
# Download a release
curl -L -o test.tar.gz "https://releases.example.com/1.0.0/binary.tar.gz"

# Extract and inspect
tar -tzf test.tar.gz | head -20

# Example output:
# yourruntime-1.0.0/
# yourruntime-1.0.0/bin/
# yourruntime-1.0.0/bin/yourruntime
```

Then configure:
```yaml
install:
  binary_path: bin/yourruntime
  strip_components: 1  # Strips "yourruntime-1.0.0/"
```

### Version Patterns

Test version extraction:

```bash
# GitHub releases often have 'v' prefix
# tag: v1.0.0 → version: 1.0.0

tag_pattern: "^v(.+)$"  # Strips 'v' prefix
```

### URL Templates

Test URL generation:
```yaml
url_template: "https://github.com/owner/repo/releases/download/{version}/binary-{os}-{arch}.tar.gz"
```

Variables:
- `{version}` → `1.0.0`
- `{os}` → `darwin`, `linux`, `windows`
- `{arch}` → `x64`, `arm64`

## Examples to Reference

Study these examples:

- **[deno.yaml](deno.yaml)** - Simple single-binary, GitHub releases
- **[go.yaml](go.yaml)** - Archive with multiple binaries, JSON API
- **[just.yaml](just.yaml)** - Tool with version in tag
- **[ripgrep.yaml](ripgrep.yaml)** - Rust tool with strip_components

## Platform Naming Reference

| Realm Platform | Common Alternatives |
|----------------|-------------------|
| `darwin-arm64` | macos-aarch64, apple-arm64 |
| `darwin-x64` | macos-x86_64, apple-amd64 |
| `linux-x64` | linux-amd64, linux-x86_64 |
| `linux-arm64` | linux-aarch64, linux-arm64 |

Map these in your manifest if needed:
```yaml
downloads:
  darwin-arm64:
    os_map:
      darwin: macos
    arch_map:
      arm64: aarch64
```

## Getting Help

- **Questions**: [Open a Discussion](https://github.com/wess/realm/discussions)
- **Issues**: [Report a Bug](https://github.com/wess/realm/issues)
- **Documentation**: [Custom Runtimes Guide](../docs/custom-runtimes.md)

## Code of Conduct

Be respectful and constructive in all interactions.

## License

Runtime manifests are licensed under MIT.
Each runtime binary is subject to its own license.
