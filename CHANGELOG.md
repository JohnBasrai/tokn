# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial workspace structure with `oauth2-client` and `oauth2-server` crates
- OAuth2 client implementation with authorization code flow
- HTTP handlers: home, login, callback, profile
- Environment-based configuration with `.env` support
- Docker Compose setup for PostgreSQL and Redis infrastructure
- Development scripts: `dev-setup.sh`, `startup.sh`, `shutdown.sh`, `test-all.sh`
- EMBP (Explicit Module Boundary Pattern) architecture
- Request tracing with configurable ANSI color output

### Changed
- None

### Deprecated
- None

### Removed
- None

### Fixed
- None

### Security
- None

## [0.1.0] - 2025-12-26

### Added
- Initial project setup
