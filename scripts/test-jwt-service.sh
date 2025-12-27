#!/usr/bin/env bash
# Manual integration tests for JWT service
# Tests all endpoints: token generation, validation, refresh, revocation

set -e

BASE_URL="http://localhost:8083"

# Colors only if stdout is a terminal AND not in Emacs
if [ -t 1 ] && [ "${EMACS:-}" != "t" ]; then
    GREEN='\033[0;32m'
    RED='\033[0;31m'
    YELLOW='\033[1;33m'
    NC='\033[0m'
else
    GREEN=''
    RED=''
    YELLOW=''
    NC=''
fi

echo "🧪 Testing JWT Service"
echo ""

# Load environment variables from .env (skip in CI - already set)
if [ -z "$CI" ]; then
    if [ -f .env ]; then
        export $(grep -v '^#' .env | xargs)
    else
        echo -e "${RED}❌ .env file not found!${NC}"
        echo "Create it from .env.example:"
        echo "  cp .env.example .env"
        echo ""
        echo "Or generate a random JWT_SECRET:"
        echo "  echo \"JWT_SECRET=\$(openssl rand -base64 32)\" >> .env"
        exit 1
    fi
fi

# Verify JWT_SECRET is set
if [ -z "$JWT_SECRET" ]; then
    echo -e "${RED}❌ JWT_SECRET not set in .env${NC}"
    echo "Add to .env:"
    echo "  echo \"JWT_SECRET=\$(openssl rand -base64 32)\" >> .env"
    exit 1
fi

# Ensure infrastructure is running (skip in CI - services already running)
if [ -z "$CI" ]; then
    echo "📦 Ensuring infrastructure is running..."
    docker compose --ansi never up -d

    # Wait for services to be healthy (max 30 seconds)
    echo "⏳ Waiting for infrastructure to be healthy..."
    for i in {1..30}; do
        REDIS_HEALTH=$(docker compose ps redis --format json 2>/dev/null | grep -o '"Health":"[^"]*"' | cut -d'"' -f4)
        POSTGRES_HEALTH=$(docker compose ps postgres --format json 2>/dev/null | grep -o '"Health":"[^"]*"' | cut -d'"' -f4)

        if [ "$REDIS_HEALTH" = "healthy" ] && [ "$POSTGRES_HEALTH" = "healthy" ]; then
            echo -e "${GREEN}✓${NC} Infrastructure ready"
            break
        fi

        if [ $i -eq 30 ]; then
            echo -e "${RED}❌ Infrastructure failed to become healthy${NC}"
            echo "Redis: $REDIS_HEALTH, Postgres: $POSTGRES_HEALTH"
            exit 1
        fi

        sleep 1
    done
    echo ""
else
    echo "📦 Running in CI - using service containers"
    echo ""
fi

# Start jwt-service in background
echo "🚀 Starting JWT service..."
JWT_SERVICE_PID=""

cleanup() {
    if [ -n "$JWT_SERVICE_PID" ]; then
        echo ""
        echo "🛑 Stopping JWT service..."
        kill $JWT_SERVICE_PID 2>/dev/null || true
        wait $JWT_SERVICE_PID 2>/dev/null || true
    fi
}

trap cleanup EXIT

# In CI: pre-build to avoid compilation timeout
if [ -n "$CI" ]; then
    echo "🔨 Building jwt-service..."
    cargo build -p jwt-service --quiet
fi

# Start service in background
cargo run -p jwt-service > /tmp/jwt-service.log 2>&1 &
JWT_SERVICE_PID=$!

# Wait for service to be ready (max 30 seconds)
echo "⏳ Waiting for service to be ready..."
for i in {1..60}; do
    if curl -s "$BASE_URL/health" > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC} Service ready"
        break
    fi
    if [ $i -eq 60 ]; then
        echo -e "${RED}❌ Service failed to start${NC}"
        echo "Logs:"
        cat /tmp/jwt-service.log
        exit 1
    fi
    sleep 0.5
done
echo ""

# Test 1: Generate tokens
echo "📝 Test 1: Token Generation"
TOKENS=$(curl -s -X POST "$BASE_URL/auth/token" \
  -H "Content-Type: application/json" \
  -d '{"user_id": "test_user", "email": "test@example.com"}')

if echo "$TOKENS" | jq -e '.access_token' > /dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} Token generation successful"
    ACCESS_TOKEN=$(echo "$TOKENS" | jq -r '.access_token')
    REFRESH_TOKEN=$(echo "$TOKENS" | jq -r '.refresh_token')
else
    echo -e "${RED}❌ Token generation failed${NC}"
    echo "$TOKENS" | jq .
    exit 1
fi
echo ""

# Test 2: Validate valid token
echo "🔍 Test 2: Token Validation (Valid)"
VALIDATION=$(curl -s -X POST "$BASE_URL/auth/validate" \
  -H "Content-Type: application/json" \
  -d "{\"token\": \"$ACCESS_TOKEN\"}")

if echo "$VALIDATION" | jq -e '.valid == true' > /dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} Valid token accepted"
else
    echo -e "${RED}❌ Valid token rejected${NC}"
    echo "$VALIDATION" | jq .
    exit 1
fi
echo ""

# Test 3: Validate invalid token
echo "🔍 Test 3: Token Validation (Invalid)"
INVALID=$(curl -s -X POST "$BASE_URL/auth/validate" \
  -H "Content-Type: application/json" \
  -d '{"token": "invalid.token.here"}')

if echo "$INVALID" | jq -e '.valid == false' > /dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} Invalid token rejected"
else
    echo -e "${RED}❌ Invalid token not rejected${NC}"
    echo "$INVALID" | jq .
    exit 1
fi
echo ""

# Test 4: Refresh token flow
echo "🔄 Test 4: Refresh Token Flow"
NEW_TOKENS=$(curl -s -X POST "$BASE_URL/auth/refresh" \
  -H "Content-Type: application/json" \
  -d "{\"refresh_token\": \"$REFRESH_TOKEN\"}")

if echo "$NEW_TOKENS" | jq -e '.access_token' > /dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} Refresh flow successful"
    NEW_ACCESS_TOKEN=$(echo "$NEW_TOKENS" | jq -r '.access_token')
    NEW_REFRESH_TOKEN=$(echo "$NEW_TOKENS" | jq -r '.refresh_token')
else
    echo -e "${RED}❌ Refresh flow failed${NC}"
    echo "$NEW_TOKENS" | jq .
    exit 1
fi
echo ""

# Test 5: Refresh token rotation (old token should fail)
echo "🔄 Test 5: Refresh Token Rotation (One-Time Use)"
OLD_REFRESH=$(curl -s -X POST "$BASE_URL/auth/refresh" \
  -H "Content-Type: application/json" \
  -d "{\"refresh_token\": \"$REFRESH_TOKEN\"}")

if echo "$OLD_REFRESH" | jq -e '.error' > /dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} Old refresh token rejected (rotation working)"
else
    echo -e "${RED}❌ Old refresh token accepted (rotation NOT working)${NC}"
    echo "$OLD_REFRESH" | jq .
    exit 1
fi
echo ""

# Test 6: Token revocation
echo "🚫 Test 6: Token Revocation"
REVOKE=$(curl -s -X POST "$BASE_URL/auth/revoke" \
  -H "Content-Type: application/json" \
  -d "{\"token\": \"$NEW_ACCESS_TOKEN\"}")

if echo "$REVOKE" | jq -e '.message' > /dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} Token revoked successfully"
else
    echo -e "${RED}❌ Revocation failed${NC}"
    echo "$REVOKE" | jq .
    exit 1
fi
echo ""

# Test 7: Validate revoked token (should fail)
echo "🔍 Test 7: Validate Revoked Token"
REVOKED=$(curl -s -X POST "$BASE_URL/auth/validate" \
  -H "Content-Type: application/json" \
  -d "{\"token\": \"$NEW_ACCESS_TOKEN\"}")

if echo "$REVOKED" | jq -e '.error | contains("revoked")' > /dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} Revoked token rejected"
else
    echo -e "${RED}❌ Revoked token not rejected${NC}"
    echo "$REVOKED" | jq .
    exit 1
fi
echo ""

# All tests passed
echo -e "${GREEN}🎉 All JWT service tests passed!${NC}"
echo ""
echo "Summary:"
echo "  ✓ Token generation"
echo "  ✓ Token validation (valid)"
echo "  ✓ Token validation (invalid)"
echo "  ✓ Refresh token flow"
echo "  ✓ Refresh token rotation"
echo "  ✓ Token revocation"
echo "  ✓ Revoked token validation"
