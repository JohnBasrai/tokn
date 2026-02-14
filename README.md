# tokn

OAuth2/OIDC and JWT authentication infrastructure demonstration in Rust.

## Overview

This repository is a Rust workspace demonstrating a **production-oriented OAuth2 / JWT authentication system**, built to explore real-world security and correctness concerns rather than just protocol compliance. It implements an end-to-end authentication flow across multiple services, with explicit trust boundaries between authorization, token issuance, and token enforcement.

The design focuses on **token lifecycle safety** (expiration, refresh rotation, revocation), **clear separation of responsibilities**, and **testable security behavior**. JWTs are treated as revocable credentials using Redis-backed invalidation, and all security-critical paths are validated via automated integration tests. Architecturally, the workspace follows Clean Architecture and the Explicit Module Boundary Pattern ([EMBP](https://github.com/JohnBasrai/architecture-patterns/blob/main/rust/embp.md)) to keep domain logic isolated from transport, storage, and framework concerns.

### Major Components

This workspace provides these three major components:

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

## Testing & Quality Assurance

**Comprehensive automated test suite** validating all authentication flows:

```bash
# Run complete JWT service test suite (10 automated tests)
./scripts/test-jwt-service.sh
```

**What's tested:**
- ✅ Token generation & validation (HS256 signing, expiration checking)
- ✅ Refresh token rotation (prevents replay attacks)
- ✅ Token revocation & blacklisting (Redis-backed)
- ✅ Protected route authentication (JWT middleware)
- ✅ Unauthorized access prevention (401 responses)
- ✅ Security edge cases (missing tokens, revoked tokens)

**Test output example:**
```
🧪 Testing JWT Service
...
✓ Token generation
✓ Token validation (valid)
✓ Token validation (invalid)
✓ Refresh token flow
✓ Refresh token rotation
✓ Token revocation
✓ Revoked token validation
✓ Protected route (valid token)
✓ Protected route (no token)
✓ Protected route (revoked token)
🎉 All JWT service tests passed!
```

**Additional testing:**
```bash
# Run all workspace tests
./scripts/test-all.sh

# Run local CI pipeline (format, clippy, tests)
./scripts/ci-local.sh
```

**Quality metrics:**
- 10 automated integration tests
- CI/CD pipeline validation
- Code formatting (`cargo fmt`)
- Linting (`cargo clippy`)
- Security-focused test scenarios

See [scripts/test-jwt-service.sh](scripts/test-jwt-service.sh) for complete test implementation.

---

## Development

**See [docs/development-setup.md](docs/development-setup.md) for:**
- Environment setup and configuration
- Running tests and local CI
- Code quality checks
- Development workflow
- Troubleshooting

**Individual service documentation:**
- [oauth2-client/README.md](oauth2-client/README.md) - OAuth2 client implementation
- [oauth2-server/README.md](oauth2-server/README.md) - Authorization server with database
- [jwt-service/README.md](jwt-service/README.md) - JWT token service with middleware

**Contributing:**
- [CONTRIBUTING.md](CONTRIBUTING.md) - Code style, documentation standards, architecture guidelines

---

## Architecture

Built following Clean Architecture and EMBP (Explicit Module Boundary Pattern).

**Key patterns:**
- Explicit module boundaries via gateway files (`mod.rs`, `lib.rs`)
- Trait-based abstractions for business logic
- Comprehensive rustdoc with RFC references
- Security-first design (token rotation, revocation, middleware)

**Code quality:**
- Automated test coverage for critical paths
- RFC-compliant implementations (OAuth2 RFC 6749, JWT RFC 7519)
- Production-ready error handling
- Logging and observability

See [EMBP documentation](https://github.com/JohnBasrai/architecture-patterns/blob/main/rust/embp.md) for architectural details.

---

## License

MIT
