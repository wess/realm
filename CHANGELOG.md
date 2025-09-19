# Changelog

All notable changes to Realm will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/wess/realm/compare/v0.1.5...HEAD
[0.1.5]: https://github.com/wess/realm/releases/tag/v0.1.5
[0.1.0]: https://github.com/wess/realm/releases/tag/v0.1.0
