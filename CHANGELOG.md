# Changelog

All notable changes to Realm will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.2.0] - 2025-01-31

### Added
- **Template variables** - Templates can now define custom variables in `template.yaml`
- Interactive prompts for template variables when not provided via CLI
- `--var KEY=VALUE` flag support for passing template variables
- Variable substitution in template files using Tera syntax
- `{{directory_name}}` placeholder for sensible defaults
- Comprehensive template variable documentation

### Changed
- All built-in templates now include `template.yaml` manifests with standard variables
- Template initialization supports both CLI flags and interactive prompts
- Skip prompts mode (`--yes`) uses template variable defaults

## [1.0.0] - 2025-01-22

### Added
- **Production-ready release** - Realm is now stable for production use
- Python runtime support with complete virtualenv isolation
- FastAPI template (React + FastAPI backend)
- Comprehensive error handling with retry logic (3 attempts, 2s backoff)
- Automatic cleanup of partial installations on failure
- Cross-platform validation (Windows, macOS Intel/ARM, Linux x64/ARM64)
- Working examples in `examples/` directory

### Changed
- Package published to crates.io as `realmenv` (binary name remains `realm`)
- Improved error messages with actionable recovery suggestions
- Enhanced documentation with installation methods and troubleshooting

### Fixed
- Documentation URLs now correctly point to GitHub README
- Installation instructions updated to use correct crate name
- Version synchronization across all project files

## [0.1.5] - 2025-01-07

### Changed
- Proxy routing now deterministically prefers exact and more specific wildcard routes for predictable forwarding.
- Deployment bundle generation sorts services and omits empty `environment` sections for cleaner Docker assets.

## [0.1.0] - 2024-01-XX

### Added
- Initial release of Realm CLI
- Virtual environment creation and activation (`realm init`, `source .venv/bin/activate`)
- Multi-runtime support (Bun and Node.js with version management)
- Process management with Foreman-like functionality (`realm start`, `realm stop`)
- Built-in HTTP proxy server with intelligent routing (`realm proxy`)
- WebSocket support for development servers (HMR, live reload)
- Configuration via `realm.yml` file
- Environment variable management (.env file support)
- Built-in project templates:
  - React + Express
  - Svelte + Fastify
  - Vue + Express
  - Next.js full-stack
- Template management (`realm templates list`, `realm create --template`)
- Deployment bundling with Docker generation (`realm bundle`)
- Comprehensive documentation (man pages, getting started guide)
- Cross-platform support (Linux, macOS, Windows)
- CI/CD with GitHub Actions
- Automated version management

### Security
- Non-root user execution in generated Docker containers
- Proper secret handling in deployment artifacts
- Secure environment variable isolation

[Unreleased]: https://github.com/wess/realm/compare/v1.2.0...HEAD
[1.2.0]: https://github.com/wess/realm/compare/v1.0.0...v1.2.0
[1.0.0]: https://github.com/wess/realm/releases/tag/v1.0.0
[0.1.5]: https://github.com/wess/realm/releases/tag/v0.1.5
[0.1.0]: https://github.com/wess/realm/releases/tag/v0.1.0
