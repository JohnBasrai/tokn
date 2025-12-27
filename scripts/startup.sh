#!/usr/bin/env bash
# Start all tokn services

set -e

echo "🚀 Starting tokn services..."
docker compose up -d
echo "✅ Infrastructure running"
echo ""
echo "Start services manually:"
echo "  cargo run -p oauth2-server  # Port 8082"
echo "  cargo run -p oauth2-client  # Port 8081"
echo ""
echo "Or run automated tests:"
echo "  ./scripts/test-jwt-service.sh  # Auto-starts jwt-service"
