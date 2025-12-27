# jwt-service

**Standalone JWT token service** demonstrating JWT generation, validation, refresh, and revocation patterns in Rust.

**Port:** 8083

---

## Overview

This service provides a complete JWT authentication system independent of OAuth2. It demonstrates:

- **Token Generation** - Create JWTs with custom claims
- **Token Validation** - Verify signatures and check expiration
- **Refresh Token Flow** - Exchange refresh tokens for new access tokens
- **Token Revocation** - Blacklist tokens before expiration
- **Protected Routes** - Middleware-based authorization

---

## Architecture

### Why JWT Service?

While `oauth2-server` handles authorization *delegation* (OAuth2 protocol), `jwt-service` focuses on *token mechanics*:

- **oauth2-server:** "Can this app access my resources?" (authorization code flow)
- **jwt-service:** "Is this token valid? Who does it belong to?" (token validation)

**In production:** oauth2-server could call jwt-service to generate/validate tokens, or jwt-service could be used standalone for API authentication.

---

## Token Types

### Access Token
- **Purpose:** Access protected resources
- **Expiry:** 15 minutes
- **Storage:** Client-side (memory, not localStorage)
- **Format:** JWT with claims

### Refresh Token
- **Purpose:** Obtain new access tokens
- **Expiry:** 7 days
- **Storage:** Redis (server-side) + httpOnly cookie (client-side)
- **Format:** UUID stored in Redis

---

## API Endpoints

### `POST /auth/token`
**Generate JWT access token**

**Request:**
```json
{
  "user_id": "user_12345",
  "email": "john@example.com"
}
```

**Response:**
```json
{
  "access_token": "eyJhbGc...",
  "token_type": "Bearer",
  "expires_in": 900,
  "refresh_token": "f47ac10b-58cc-4372-a567-0e02b2c3d479"
}
```

---

### `POST /auth/validate`
**Validate JWT token**

**Request:**
```json
{
  "token": "eyJhbGc..."
}
```

**Response (valid):**
```json
{
  "valid": true,
  "claims": {
    "sub": "user_12345",
    "email": "john@example.com",
    "exp": 1703001234,
    "iat": 1703000334
  }
}
```

**Response (invalid):**
```json
{
  "valid": false,
  "error": "Token expired"
}
```

---

### `POST /auth/refresh`
**Exchange refresh token for new access token**

**Request:**
```json
{
  "refresh_token": "f47ac10b-58cc-4372-a567-0e02b2c3d479"
}
```

**Response:**
```json
{
  "access_token": "eyJhbGc...",
  "token_type": "Bearer",
  "expires_in": 900
}
```

**Security:** Old refresh token is invalidated (rotation).

---

### `POST /auth/revoke`
**Revoke (blacklist) a token**

**Request:**
```json
{
  "token": "eyJhbGc..."
}
```

**Response:**
```json
{
  "revoked": true
}
```

**Implementation:** Token JTI (JWT ID) added to Redis blacklist with TTL = remaining token lifetime.

---

### `GET /protected`
**Demo protected endpoint requiring valid JWT**

**Headers:**
```
Authorization: Bearer eyJhbGc...
```

**Response:**
```json
{
  "message": "Access granted",
  "user_id": "user_12345",
  "email": "john@example.com"
}
```

---

## Token Claims

### Standard Claims (RFC 7519)

```rust
#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // Subject (user ID)
    pub email: String,      // Custom claim
    pub exp: usize,         // Expiration time (Unix timestamp)
    pub iat: usize,         // Issued at time
    pub jti: String,        // JWT ID (for revocation)
}
```

---

## Security Design

### Signing Algorithm
- **Default:** HS256 (HMAC SHA-256)
- **Secret:** 256-bit random key (environment variable)
- **Future:** RS256 support (asymmetric keys)

### Token Expiration
- Access token: **15 minutes** (balance security vs. UX)
- Refresh token: **7 days** (requires re-authentication after)

### Revocation Strategy
- **Blacklist in Redis** with TTL = token expiry
- Trade-off: Adds database lookup, but enables instant revocation

### Refresh Token Rotation
- Each refresh invalidates the old token
- Prevents replay attacks if refresh token is stolen

---

## Configuration

### Environment Variables

```bash
# Server
JWT_SERVICE_HOST=127.0.0.1
JWT_SERVICE_PORT=8083

# JWT
JWT_SECRET=your-256-bit-secret-key-here

# Redis
REDIS_URL=redis://127.0.0.1:6379
```

### Generate JWT Secret

```bash
openssl rand -base64 32
```

**⚠️ Never commit secrets to git!**

---

## Running the Service

### Prerequisites
- Docker and Docker Compose (for infrastructure)
- Redis and PostgreSQL will be started via Docker Compose

### Start Infrastructure

```bash
# From workspace root
docker compose up -d

# Check services are healthy
docker compose ps
```

### Start JWT Service

**Option 1: Run on host (development)**
```bash
# From workspace root
JWT_SECRET=$(openssl rand -base64 32) cargo run --bin jwt-service
```

**Option 2: Run via cargo from jwt-service directory**
```bash
cd jwt-service
JWT_SECRET=$(openssl rand -base64 32) cargo run
```

Service starts on `http://127.0.0.1:8083`

### Stop Services

```bash
# Stop infrastructure
docker compose down

# Or stop and remove volumes
docker compose down -v
```

---

## Testing

### Unit Tests
```bash
cargo test --package jwt-service
```

### Integration Tests
```bash
# Generate token
curl -X POST http://localhost:8083/auth/token \
  -H "Content-Type: application/json" \
  -d '{"user_id": "test_user", "email": "test@example.com"}'

# Validate token
curl -X POST http://localhost:8083/auth/validate \
  -H "Content-Type: application/json" \
  -d '{"token": "eyJhbGc..."}'

# Access protected endpoint
curl http://localhost:8083/protected \
  -H "Authorization: Bearer eyJhbGc..."
```

---

## JWT vs OAuth2: When to Use Each

| Scenario | Use OAuth2 | Use JWT |
|----------|------------|---------|
| Third-party app authorization | ✅ | ❌ |
| First-party API authentication | ❌ | ✅ |
| Social login (Google, GitHub) | ✅ | ❌ |
| Microservice auth | ❌ | ✅ |
| Stateless API tokens | ❌ | ✅ |
| Granular scopes/permissions | ✅ | Both* |

\* JWTs can contain scopes, but OAuth2 provides the delegation framework.

---

## Comparison with oauth2-server

| Feature | oauth2-server | jwt-service |
|---------|---------------|-------------|
| **Protocol** | OAuth2 / OIDC | JWT (RFC 7519) |
| **Token Format** | Opaque UUID | Signed JWT |
| **Validation** | Database lookup | Signature verification |
| **Use Case** | App authorization | API authentication |
| **Stateless** | No (stores codes/tokens) | Yes (except revocation) |

**Can they work together?** Yes! oauth2-server could generate JWTs as access tokens instead of UUIDs.

---

## Implementation Notes

### EMBP Architecture
- Follows Explicit Module Boundary Pattern
- `src/handlers/mod.rs` defines public API
- Sibling imports via `super::`

### Error Handling
- Uses `anyhow::Result` for ergonomic error handling
- Detailed error messages for debugging
- Security: Don't leak token details in production errors

### Middleware
- `AuthMiddleware` extracts and validates JWT from `Authorization` header
- Adds `Claims` to request extensions for downstream handlers

---

## Future Enhancements

- [ ] RS256 (RSA) support for asymmetric signing
- [ ] JWK (JSON Web Key) endpoint for public key distribution
- [ ] Token introspection endpoint (RFC 7662)
- [ ] Scope-based authorization
- [ ] Rate limiting on token generation
- [ ] Audit logging

---

## References

- [JWT Primer](../docs/jwt-primer.md) - Theory and concepts
- [EMBP Pattern](https://github.com/JohnBasrai/architecture-patterns/blob/main/rust/embp.md) - Explicit Module Boundary Pattern
- [RFC 7519](https://datatracker.ietf.org/doc/html/rfc7519) - JWT Specification
- [jsonwebtoken docs](https://docs.rs/jsonwebtoken/) - Rust crate documentation
- [jwt.io](https://jwt.io) - JWT debugger

---

## License

MIT
