#!/usr/bin/env bash
# Run all tests for tokn workspace

set -e

echo "🧪 Running tokn test suite..."

# Ensure infrastructure is running
docker compose up -d

# Wait for services
until docker compose exec postgres pg_isready -U postgres > /dev/null 2>&1; do
    sleep 1
done

# Run tests
SQLX_OFFLINE=true cargo test --workspace --all-features

echo "✅ All tests passed!"
