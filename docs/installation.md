# Installation

Install Realm using one of the methods below.

## Quick Install (macOS/Linux)

Download and install with a single command:

```bash
curl -sSfL https://github.com/wess/realm/releases/latest/download/install.sh | bash
```

This installs realm to `/usr/local/bin`.

## From crates.io (Recommended)

If you have Rust installed:

```bash
cargo install realmenv
```

The binary will be available as `realm`.

## Pre-built Binaries

Download the latest release for your platform:

### macOS

**Intel (x86_64)**:
```bash
curl -L -o realm https://github.com/wess/realm/releases/latest/download/realm-macos-amd64
chmod +x realm
sudo mv realm /usr/local/bin/
```

**Apple Silicon (ARM64)**:
```bash
curl -L -o realm https://github.com/wess/realm/releases/latest/download/realm-macos-arm64
chmod +x realm
sudo mv realm /usr/local/bin/
```

### Linux

**x86_64**:
```bash
curl -L -o realm https://github.com/wess/realm/releases/latest/download/realm-linux-amd64
chmod +x realm
sudo mv realm /usr/local/bin/
```

**ARM64**:
```bash
curl -L -o realm https://github.com/wess/realm/releases/latest/download/realm-linux-arm64
chmod +x realm
sudo mv realm /usr/local/bin/
```

### Windows

Download [realm-windows-amd64.exe](https://github.com/wess/realm/releases/latest/download/realm-windows-amd64.exe) and add it to your PATH.

## From Source

### Prerequisites

- Rust 1.75 or later
- Git

### Build Steps

```bash
# Clone repository
git clone https://github.com/wess/realm
cd realm

# Build and install
cargo install --path .
```

The binary will be installed to `~/.cargo/bin/realm`.

## Verify Installation

Check that realm is installed correctly:

```bash
realm --version
```

You should see:
```
realm 1.2.0
```

## Shell Completions

Generate completions for your shell:

### Bash

```bash
realm completions bash | sudo tee /etc/bash_completion.d/realm
```

### Zsh

```bash
realm completions zsh > ~/.zfunc/_realm
# Add to ~/.zshrc if not already present:
# fpath=(~/.zfunc $fpath)
# autoload -Uz compinit && compinit
```

### Fish

```bash
realm completions fish > ~/.config/fish/completions/realm.fish
```

## System Requirements

- **Operating System**: macOS 10.15+, Linux (any modern distro), Windows 10+
- **Disk Space**: ~10MB for realm binary, additional space for runtimes
- **Network**: Required for downloading runtimes and templates

## Troubleshooting

### "command not found: realm"

The binary is not in your PATH. Either:
- Use the full path: `~/.cargo/bin/realm`
- Add to PATH: `export PATH="$HOME/.cargo/bin:$PATH"` (add to `~/.bashrc` or `~/.zshrc`)

### Permission denied when installing to /usr/local/bin

Use `sudo` for the move command:
```bash
sudo mv realm /usr/local/bin/
```

### Rust not installed (for cargo install)

Install Rust via rustup:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Upgrading

### Via cargo

```bash
cargo install realmenv --force
```

### Via binary

Download the latest release and replace your existing binary.

### Check for updates

```bash
# Current version
realm --version

# Check latest release at:
# https://github.com/wess/realm/releases/latest
```

## Next Steps

- [Quick Start Guide](quickstart.md) - Create your first realm environment
- [Commands Reference](commands.md) - Learn available commands
