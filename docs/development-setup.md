# Development Environment Setup

This guide covers the tools and setup needed to contribute to tokn (rust-auth-infrastructure).

## Prerequisites

### Required Tools

- **Rust** - Install via [rustup](https://rustup.rs/)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **Docker** - Required for PostgreSQL and Redis services
  - [Docker Desktop](https://www.docker.com/products/docker-desktop/) (macOS/Windows)
  - [Docker Engine](https://docs.docker.com/engine/install/) (Linux)

- **Docker Compose** - Usually included with Docker Desktop
  ```bash
  # Verify installation
  docker compose version
  ```

## Local Testing with `act`

This project uses [act](https://github.com/nektos/act) to run GitHub Actions workflows locally.

### Installing `act`

**macOS (Homebrew):**
```bash
brew install act
```

**Linux:**
```bash
# Download install script
curl https://raw.githubusercontent.com/nektos/act/master/install.sh > act-install.sh
chmod +x act-install.sh

# Install to /usr/local/bin (requires sudo)
sudo ./act-install.sh -b /usr/local/bin

# Clean up
rm act-install.sh
```

**Verify installation:**
```bash
act --version
```

### Upgrading `act`

**macOS:**
```bash
brew upgrade act
```

**Linux:**
```bash
# Download and run install script (checks if upgrade needed)
curl https://raw.githubusercontent.com/nektos/act/master/install.sh > act-install.sh
chmod +x act-install.sh
sudo ./act-install.sh -b /usr/local/bin
rm act-install.sh
```

The install script automatically checks versions and skips installation if already up to date.

### Running Local CI

```bash
# Full CI run (same checks as GitHub Actions)
./scripts/ci-local.sh

# Save output to log file
./scripts/ci-local.sh |& tee ci-local.log
```

**First run:** Downloads Docker images and may take several minutes.

## Environment Configuration

### Create `.env` file

Copy the example file and customize:

```bash
cp .env.example .env
```

**Required variables:**
- `JWT_SECRET` - Must be at least 32 characters
- `DATABASE_URL` - PostgreSQL connection string
- `REDIS_URL` - Redis connection string

**Generate a secure JWT_SECRET:**
```bash
echo "JWT_SECRET=$(openssl rand -base64 32)" >> .env
```

### Start Infrastructure

```bash
# Start PostgreSQL and Redis
./scripts/startup.sh

# Stop services
./scripts/shutdown.sh
```

## Running Tests

### Integration Tests

```bash
# JWT service integration tests (auto-starts service)
./scripts/test-jwt-service.sh

# All workspace tests
./scripts/test-all.sh
```

**Expected output from test-jwt-service.sh:**
```
🧪 Testing JWT Service

📦 Ensuring infrastructure is running...

🚀 Starting JWT service...
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.14s
     Running `target/debug/jwt-service`
2025-12-27T19:12:53.979703Z  INFO jwt_service: Starting JWT service on 127.0.0.1:8083
2025-12-27T19:12:53.980530Z  INFO jwt_service: Connected to Redis at redis://127.0.0.1:6379
2025-12-27T19:12:53.980711Z  INFO jwt_service: JWT service listening on 127.0.0.1:8083
2025-12-27T19:12:53.980722Z  INFO jwt_service: Endpoints:
2025-12-27T19:12:53.980728Z  INFO jwt_service:   POST /auth/token - Generate JWT and refresh tokens
2025-12-27T19:12:53.980734Z  INFO jwt_service:   POST /auth/validate - Validate JWT token
2025-12-27T19:12:53.980740Z  INFO jwt_service:   POST /auth/refresh - Refresh access token
2025-12-27T19:12:53.980746Z  INFO jwt_service:   POST /auth/revoke - Revoke (blacklist) JWT token

⏳ Waiting for service to be ready...
✓ Service ready

📝 Test 1: Token Generation
✓ Token generation successful
...
🎉 All JWT service tests passed!
```

### Unit Tests

```bash
# All tests
SQLX_OFFLINE=true cargo test

# Specific crate
SQLX_OFFLINE=true cargo test -p oauth2-client
SQLX_OFFLINE=true cargo test -p oauth2-server
SQLX_OFFLINE=true cargo test -p jwt-service
```

**Note:** See [docs/sqlx-offline-mode-howto.md](sqlx-offline-mode-howto.md) for SQLx offline mode details.

## Code Quality Checks

```bash
# Format code
cargo fmt

# Check formatting (CI uses this)
cargo fmt --check

# Run clippy linter
SQLX_OFFLINE=true cargo clippy --all-targets --all-features --no-deps -- -D warnings

# Check for unused dependencies
cargo machete

# Security audit
cargo audit
```

## Development Workflow

1. **Start infrastructure:** `./scripts/startup.sh`
2. **Create feature branch:** `git checkout -b feat/my-feature`
3. **Make changes and test:** `./scripts/test-jwt-service.sh`
4. **Format and lint:** `cargo fmt && cargo clippy`
5. **Run full CI locally:** `./scripts/ci-local.sh`
6. **Commit:** Follow conventional commits
7. **Push and create PR**

## IDE Setup (Optional)

### VS Code

Recommended extensions:
- `rust-analyzer` - Rust language server
- `Even Better TOML` - TOML syntax highlighting
- `crates` - Cargo.toml dependency management

### Emacs

The test scripts detect Emacs environment and disable ANSI colors automatically.

### Other IDEs

Any IDE with Rust support via rust-analyzer will work well.

## Troubleshooting

### Port Already in Use

If you see "port already allocated" errors:
```bash
./scripts/shutdown.sh
docker compose down
./scripts/startup.sh
```

### `act` Issues

**Cache warnings:** These are safe to ignore:
```
Cache not found for input keys: Linux-cargo-git-
```

**Permission denied:** Run without `sudo` - the install script handles permissions.

### SQLx Offline Mode

If you see SQLx errors, ensure `SQLX_OFFLINE=true` is set:
```bash
export SQLX_OFFLINE=true
cargo test
```

## Next Steps

- Read [CONTRIBUTING.md](../CONTRIBUTING.md) for code style and architecture guidelines
- Review [docs/jwt-primer.md](jwt-primer.md) for JWT implementation details
- Check [docs/sqlx-offline-mode-howto.md](sqlx-offline-mode-howto.md) for database setup

## Getting Help

- Check existing issues on GitHub
- Review documentation in `docs/`
- Look at test files for usage examples
