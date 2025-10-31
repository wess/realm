# Runtime Plugin Architecture

Architecture change to make Realm extensible with custom runtimes without modifying core code.

## Motivation

**Problem**: Runtimes (Bun, Node, Python) are hardcoded in the codebase. Adding new runtimes requires:
- Modifying core Rust code
- Understanding complex version fetching logic
- Implementing platform-specific download URLs
- Handling extraction and installation

**Solution**: Plugin-based architecture where runtimes can be defined declaratively via YAML manifests or programmatically via a trait.

## Architecture Overview

```
~/.realm/
├── runtimes/              # Downloaded runtime binaries
│   ├── bun-1.1.34/
│   ├── node-20.18.0/
│   └── python-3.12.6/
├── runtimes-config/       # Runtime manifests (NEW)
│   ├── deno.yaml
│   ├── go.yaml
│   ├── ruby.yaml
│   └── zig.yaml
└── cache/
    └── versions.json
```

## Components

### 1. RuntimeProvider Trait

Core trait that all runtimes must implement:

```rust
pub trait RuntimeProvider: Send + Sync {
    fn name(&self) -> &str;
    fn aliases(&self) -> Vec<&str>;
    async fn list_versions(&self) -> Result<Vec<String>>;
    async fn get_artifact(&self, version: &str, platform: &PlatformInfo) -> Result<RuntimeArtifact>;
    async fn install_artifact(&self, artifact_data: &[u8], artifact: &RuntimeArtifact, install_dir: &PathBuf) -> Result<()>;
    // ... more methods
}
```

**File**: `src/runtime/provider.rs`

### 2. Runtime Manifest

YAML-based declarative runtime configuration:

```yaml
runtime:
  name: deno
  display_name: Deno
  aliases: []
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

**File**: `src/runtime/manifest.rs`

### 3. DeclarativeProvider

Provider implementation that reads manifests and handles downloads/installation:

```rust
pub struct DeclarativeProvider {
    manifest: RuntimeManifest,
    client: Client,
}

impl RuntimeProvider for DeclarativeProvider {
    // Implements all RuntimeProvider methods based on manifest
}
```

**File**: `src/runtime/declarative.rs`

### 4. RuntimeRegistry

Central registry for discovering and managing runtime providers:

```rust
pub struct RuntimeRegistry {
    providers: HashMap<String, Arc<dyn RuntimeProvider>>,
}

impl RuntimeRegistry {
    pub fn register(&mut self, provider: Arc<dyn RuntimeProvider>);
    pub fn get(&self, name: &str) -> Option<Arc<dyn RuntimeProvider>>;
    pub async fn discover_runtimes(&mut self) -> Result<()>;
}
```

**File**: `src/runtime/registry.rs`

## Usage Flow

### User Perspective

1. **Add Custom Runtime**:
   ```bash
   cat > ~/.realm/runtimes-config/deno.yaml << 'EOF'
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
   EOF
   ```

2. **Use It**:
   ```bash
   realm init --runtime=deno
   realm init --runtime=deno@1.40.0
   ```

### Implementation Flow

1. **Initialization**:
   - RuntimeRegistry created
   - Built-in providers registered (Bun, Node, Python)
   - `discover_runtimes()` called

2. **Discovery**:
   - Scans `~/.realm/runtimes-config/` for `.yaml` files
   - Parses each manifest
   - Creates DeclarativeProvider for each
   - Registers provider (including aliases)

3. **Runtime Selection**:
   - User runs `realm init --runtime=deno`
   - Registry looks up "deno" provider
   - Returns `Arc<dyn RuntimeProvider>`

4. **Installation**:
   - `list_versions()` - Fetch available versions
   - `resolve_latest()` - Resolve "latest" to actual version
   - `get_artifact()` - Get download URL for platform
   - Download artifact with progress bar
   - `install_artifact()` - Extract to `~/.realm/runtimes/`
   - `post_install()` - Run post-install commands
   - Create symlinks in `.venv/bin/`

## Manifest Specification

### Version Discovery

**GitHub Releases**:
```yaml
versions:
  type: github
  repo: owner/repo
  tag_pattern: "^v(.+)$"  # Extract version from tag
```

**JSON API**:
```yaml
versions:
  type: api
  url: "https://api.example.com/versions"
  json_path: data.versions
```

**Static List**:
```yaml
versions:
  type: static
  versions:
    - "1.0.0"
    - "1.1.0"
```

### Platform Downloads

```yaml
downloads:
  darwin-arm64:
    url_template: "https://example.com/{version}/binary-{os}-{arch}.tar.gz"
    format: tar.gz
    checksum_url: "https://example.com/{version}/checksums.txt"
    checksum_algo: sha256

    # Optional: Name mappings
    os_map:
      darwin: macos
    arch_map:
      arm64: aarch64
```

**Template Variables**:
- `{version}` - Version being installed
- `{os}` - Operating system (darwin, linux, windows)
- `{arch}` - Architecture (x64, arm64)

### Installation

```yaml
install:
  binary_path: bin/myruntime
  additional_binaries:
    - tool1
    - tool2
  strip_components: 1
  post_install_commands:
    - "chmod +x bin/*"
```

### Environment

```yaml
environment:
  requires_isolation: false
  vars:
    MY_VAR: "{install_dir}/path"
  isolation_commands:
    - "setup-venv.sh"
```

## Example Runtimes

Complete examples in `examples/runtimes/`:
- `deno.yaml` - Deno runtime
- `go.yaml` - Go programming language

## Migration Path

### Phase 1: Plugin Infrastructure (Current)
- ✓ RuntimeProvider trait
- ✓ RuntimeManifest structure
- ✓ DeclarativeProvider implementation
- ✓ RuntimeRegistry with discovery
- ✓ Documentation

### Phase 2: Built-in Runtime Migration (Future)
- Migrate Bun to DeclarativeProvider
- Migrate Node.js to DeclarativeProvider
- Migrate Python to DeclarativeProvider
- Remove hardcoded runtime logic

### Phase 3: Community Ecosystem (Future)
- Create realm-runtimes repository
- Community-contributed runtime manifests
- Auto-discovery from remote sources

## Benefits

1. **Extensibility**: Add new runtimes without touching core code
2. **Maintainability**: Runtime logic isolated in manifests
3. **Community**: Users can share runtime configs
4. **Flexibility**: Both declarative and programmatic options
5. **Testability**: Easy to test with static version lists

## File Changes

### New Files
- `src/runtime/provider.rs` - RuntimeProvider trait
- `src/runtime/manifest.rs` - YAML manifest structures
- `src/runtime/declarative.rs` - Declarative provider implementation
- `src/runtime/registry.rs` - Runtime registry
- `examples/runtimes/deno.yaml` - Deno example
- `examples/runtimes/go.yaml` - Go example
- `docs/custom-runtimes.md` - Complete documentation

### Modified Files (Future)
- `src/runtime/manager.rs` - Use RuntimeRegistry
- `src/runtime/types.rs` - Remove hardcoded enum
- `src/cli/mod.rs` - Use registry for runtime lookup
- `Cargo.toml` - Add dependencies (async-trait, regex if needed)

## Dependencies

### Required
- `async-trait` - Async trait support
- `serde` + `serde_yaml` - YAML parsing (already have)
- `reqwest` - HTTP client (already have)

### Optional
- `regex` - For tag_pattern matching
- `jsonpath` - For complex JSON path queries

## Testing Strategy

1. **Unit Tests**:
   - Manifest parsing
   - URL template substitution
   - Platform detection

2. **Integration Tests**:
   - Load manifest from file
   - Parse GitHub releases
   - Download and extract (mocked)

3. **Example Manifests**:
   - Provide working examples
   - Test against real APIs

## Next Steps

1. ✅ Design and document architecture
2. ✅ Implement core traits and structures
3. ✅ Create example manifests
4. ✅ Write documentation
5. ⏭️ Add to Cargo.toml (`async-trait`, `regex`)
6. ⏭️ Wire into existing RuntimeManager
7. ⏭️ Test with example runtimes
8. ⏭️ Migrate built-in runtimes

## Backward Compatibility

- Existing hardcoded runtimes continue to work
- No breaking changes to user-facing API
- `realm init --runtime=bun` still works
- New manifests opt-in via `~/.realm/runtimes-config/`

## Security Considerations

1. **URL Validation**: Only HTTPS from trusted domains
2. **Checksum Verification**: Optional but recommended
3. **Manifest Validation**: Schema validation on load
4. **Sandboxing**: Post-install commands run in restricted env
5. **Trust Model**: Users explicitly install manifests

## Performance

- **Lazy Loading**: Manifests loaded on-demand
- **Caching**: Version lists cached (24hr TTL)
- **Parallel Downloads**: Same as existing
- **Registry Overhead**: Minimal (HashMap lookup)

## Future Enhancements

1. **Remote Manifests**: Auto-fetch from git repos
2. **Manifest Signing**: GPG-signed manifests for trust
3. **Dependency Resolution**: Runtimes depending on other runtimes
4. **Plugin Hooks**: Pre/post install hooks
5. **GUI**: Visual runtime browser/installer
6. **Version Constraints**: Semantic versioning support
