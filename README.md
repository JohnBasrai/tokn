# tokn

OAuth2 authentication infrastructure demonstration in Rust.

## Overview

This workspace demonstrates OAuth2 authorization code flow with:

- **oauth2-client** (port 8081) - Demo application using OAuth2 authentication
- **oauth2-server** (port 8082) - OAuth2 authorization server implementation

## Quick Start

```bash
# Start infrastructure (PostgreSQL + Redis)
docker compose up -d

# Run oauth2-server
cargo run -p oauth2-server

# Run oauth2-client (in another terminal)
cargo run -p oauth2-client

# Visit http://localhost:8081
```

## Prerequisites
- Rust toolchain
- Docker & Docker Compose
- sqlx-cli: `cargo install sqlx-cli --no-default-features --features postgres`

## Development

See individual service READMEs:
- [oauth2-client/README.md](oauth2-client/README.md)
- [oauth2-server/README.md](oauth2-server/README.md)

## Architecture

Built following Clean Architecture and EMBP (Explicit Module Boundary Pattern).

## License

MIT
