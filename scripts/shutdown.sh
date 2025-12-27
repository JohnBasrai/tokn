#!/usr/bin/env bash
# Stop all tokn services

set -e

echo "🛑 Stopping tokn services..."
docker compose --ansi never down
echo "✅ Services stopped"
