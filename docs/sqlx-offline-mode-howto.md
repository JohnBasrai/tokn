# SQLx Offline Mode

## Overview

SQLx's `query!` macro validates queries against a live database at compile-time. For CI/CD environments where databases aren't available during builds, SQLx provides "offline mode" using pre-generated query metadata.

## Generating Query Metadata

**Prerequisites:**
- PostgreSQL running with migrations applied
- Database accessible at `DATABASE_URL`

**Steps:**

1. Start database and apply migrations:
```bash
   docker compose up -d postgres
   cd oauth2-server
   sqlx migrate run --database-url "postgresql://postgres:postgres@localhost:5432/tokn_db"
```

2. Generate query metadata:
```bash
   cargo sqlx prepare --workspace -- --lib
```

3. Verify `.sqlx/` directory created:
```bash
   cd ../ # Return to workspace root
   ls -la oauth2-server/.sqlx/
   # Should contain query-*.json files at workspace root
```

4. Commit the metadata:
```bash
   git add .sqlx/
   git commit -m "chore: add sqlx offline query metadata"
```

## When to Regenerate

Regenerate query metadata whenever you:
- Add new `sqlx::query!` or `sqlx::query_as!` calls
- Modify existing queries
- Change database schema (migrations)

## Using Offline Mode

Set environment variable:
```bash
export SQLX_OFFLINE=true
cargo build
```

Or per-command:
```bash
SQLX_OFFLINE=true cargo clippy
```

## CI/CD Configuration

GitHub Actions workflow already configured to use offline mode:
```yaml
env:
  SQLX_OFFLINE: true
```

## Troubleshooting

**"query not found in offline cache":**
- Regenerate metadata: `cargo sqlx prepare --workspace -- --lib`
- Ensure `.sqlx/` is committed to git

**Metadata out of sync:**
- Delete `.sqlx/` directory
- Regenerate from scratch with current schema

## Reference
- [SQLx Offline Mode Docs](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md#enable-building-in-offline-mode-with-query)

