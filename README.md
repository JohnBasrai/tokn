# tokn

OAuth2/OIDC and JWT authentication infrastructure demonstration in Rust.

## Overview

This workspace demonstrates authentication patterns with:

- **oauth2-client** (port 8081) - Demo application using OAuth2 authentication
- **oauth2-server** (port 8082) - OAuth2 authorization server implementation
- **jwt-service** (port 8083) - Standalone JWT token service

---

## Prerequisites

- Rust toolchain (install via [rustup](https://rustup.rs/))
- Docker & Docker Compose
- `.env` file (copy from `.env.example`)

**For detailed environment setup, see [docs/development-setup.md](docs/development-setup.md)**

---

## Quick Start

**Note:** `.env.example` includes a demo JWT_SECRET. For production, generate a secure secret (see [Prerequisites](#prerequisites)).

```bash
# Copy example config
cp .env.example .env

# Start infrastructure (PostgreSQL + Redis)
docker compose up -d

# Run oauth2-server
cargo run -p oauth2-server

# Run oauth2-client (in another terminal)
cargo run -p oauth2-client

# Run jwt-service (in another terminal)
cargo run -p jwt-service

# Test endpoints:
#   OAuth2 client: http://localhost:8081
#   JWT health:    http://localhost:8083/health
#   OAuth2 server: http://localhost:8082 (API - see endpoints in logs)
```

---

## Testing

```bash
# Run JWT service integration tests
./scripts/test-jwt-service.sh

# Run all workspace tests
./scripts/test-all.sh
```

---

## Development

**See [docs/development-setup.md](docs/development-setup.md) for:**
- Environment setup and configuration
- Running tests and local CI
- Code quality checks
- Development workflow
- Troubleshooting

**Individual service documentation:**
- [oauth2-client/README.md](oauth2-client/README.md)
- [oauth2-server/README.md](oauth2-server/README.md)
- [jwt-service/README.md](jwt-service/README.md)

**Contributing:**
- [CONTRIBUTING.md](CONTRIBUTING.md) - Code style, documentation standards, architecture guidelines

---

## Architecture

Built following Clean Architecture and EMBP (Explicit Module Boundary Pattern).

**Key patterns:**
- Explicit module boundaries via gateway files (`mod.rs`, `lib.rs`)
- Trait-based abstractions for business logic
- Comprehensive rustdoc with RFC references

See [EMBP documentation](https://github.com/JohnBasrai/architecture-patterns/blob/main/rust/embp.md) for details.

---

## License

MIT
