# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Complete OAuth2 authorization server implementation
- Authorization endpoint with consent page (GET/POST /oauth/authorize)
- Token exchange endpoint (POST /oauth/token)
- User info endpoint (GET /oauth/userinfo)
- Database migrations for OAuth2 schema (clients, users, codes, tokens)
- Seed data: demo user and client credentials
- Manual request body deserialization for better error handling
- Comprehensive validation and error responses
- Initial workspace structure with `oauth2-client` and `oauth2-server` crates
- OAuth2 client implementation with authorization code flow
- HTTP handlers: home, login, callback, profile
- Environment-based configuration with `.env` support
- Docker Compose setup for PostgreSQL and Redis infrastructure
- Development scripts: `dev-setup.sh`, `startup.sh`, `shutdown.sh`, `test-all.sh`
- EMBP (Explicit Module Boundary Pattern) architecture
- Request tracing with configurable ANSI color output

### Changed
- OAuth2 client now uses RequestBody auth type
- Client callback handler fetches and displays user info
- Enhanced logging for token exchange debugging

### Completed
- Full OAuth2 authorization code flow working end-to-end

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
