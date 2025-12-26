#!/usr/bin/env bash
# Development environment setup for tokn

set -e

echo "🚀 Setting up tokn development environment..."

# Check for required tools
command -v docker >/dev/null 2>&1 || { echo "❌ Docker required"; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo "❌ Cargo required"; exit 1; }
command -v psql >/dev/null 2>&1 || { echo "❌ PostgreSQL client required"; exit 1; }

# Start infrastructure
echo "📦 Starting Docker containers..."
docker compose up -d

# Wait for services
echo "⏳ Waiting for PostgreSQL..."
until docker compose exec postgres pg_isready -U postgres > /dev/null 2>&1; do
    sleep 1
done

echo "⏳ Waiting for Redis..."
until docker compose exec redis redis-cli ping > /dev/null 2>&1; do
    sleep 1
done

# Run migrations (when we add them)
# echo "🗄️  Running migrations..."
# cargo sqlx migrate run --database-url "postgresql://postgres:postgres@localhost:5432/tokn_db"

echo "✅ Development environment ready!"
echo ""
echo "Next steps:"
echo "  cargo run -p oauth2-server"
echo "  cargo run -p oauth2-client"
