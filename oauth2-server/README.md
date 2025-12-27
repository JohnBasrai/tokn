# oauth2-server

**OAuth2 authorization server** implementing the authorization code flow with PostgreSQL storage for codes, tokens, and clients.

**Port:** 8082

---

## Overview

This service demonstrates an **OAuth2 authorization server** (provider). It showcases:

- **Authorization Endpoint** - User consent UI for resource access
- **Token Exchange** - Exchange authorization codes for access tokens
- **User Info Endpoint** - Protected resource endpoint
- **Client Registration** - OAuth2 client credentials management
- **PostgreSQL Storage** - Persistent storage for codes and tokens
- **Argon2 Password Hashing** - Secure password storage for demo users

---

## Architecture

### OAuth2 Roles

In the OAuth2 ecosystem, this service is the **Authorization Server**:

- **Resource Owner** - End user (authenticated via username/password)
- **Client** - oauth2-client (registered app requesting access)
- **Authorization Server** - **This service** (oauth2-server)
- **Resource Server** - Also this service (provides `/oauth/userinfo`)

### Authorization Code Flow

```
┌─────────────┐                                      ┌─────────────┐
│   Client    │                                      │   Server    │
│ (oauth2-    │                                      │ (oauth2-    │
│  client)    │                                      │  server)    │
└──────┬──────┘                                      └──────┬──────┘
       │                                                    │
       │  1. GET /oauth/authorize?client_id=...&           │
       │     redirect_uri=...&state=...                    │
       │──────────────────────────────────────────────────>│
       │                                                    │
       │  2. Show consent page (if not authenticated,      │
       │     prompt for login first)                       │
       │<──────────────────────────────────────────────────│
       │                                                    │
       │  3. User approves (POST /oauth/authorize)         │
       │──────────────────────────────────────────────────>│
       │                                                    │
       │  4. Redirect to callback with code                │
       │     Location: redirect_uri?code=xyz&state=...     │
       │<──────────────────────────────────────────────────│
       │                                                    │
       │  5. POST /oauth/token                             │
       │     grant_type=authorization_code&code=xyz        │
       │──────────────────────────────────────────────────>│
       │                                                    │
       │  6. Return access_token                           │
       │     {"access_token": "...", "token_type": ...}    │
       │<──────────────────────────────────────────────────│
       │                                                    │
       │  7. GET /oauth/userinfo                           │
       │     Authorization: Bearer access_token            │
       │──────────────────────────────────────────────────>│
       │                                                    │
       │  8. Return user profile                           │
       │     {"user_id": "...", "email": "...", ...}       │
       │<──────────────────────────────────────────────────│
```

---

## API Endpoints

### `GET /oauth/authorize`
**Authorization endpoint - initial request**

Initiates the OAuth2 authorization code flow. Shows consent page if user is authenticated, otherwise shows login form.

**Query Parameters:**
- `response_type` - Must be "code" (authorization code flow)
- `client_id` - Registered client identifier
- `redirect_uri` - Where to send user after authorization
- `state` - Client-provided CSRF token (optional but recommended)
- `scope` - Requested permissions (optional)

**Example:**
```
GET /oauth/authorize?
  response_type=code&
  client_id=demo_client&
  redirect_uri=http://localhost:8081/callback&
  state=random-csrf-token
```

**Response:** HTML consent page

---

### `POST /oauth/authorize`
**Authorization endpoint - user approval**

Processes user consent decision.

**Form Data:**
- `client_id` - Client identifier
- `redirect_uri` - Callback URI
- `state` - CSRF token from initial request
- `user_id` - Authenticated user (from session)
- `approved` - "true" if user consented

**Success Response:** 302 Redirect
```
Location: http://localhost:8081/callback?code=ABC123&state=random-csrf-token
```

**Error Response:** 302 Redirect
```
Location: http://localhost:8081/callback?error=access_denied&state=random-csrf-token
```

---

### `POST /oauth/token`
**Token endpoint - exchange code for token**

Exchanges authorization code for access token.

**Request:**
```bash
POST /oauth/token
Content-Type: application/x-www-form-urlencoded

grant_type=authorization_code&
code=ABC123&
redirect_uri=http://localhost:8081/callback&
client_id=demo_client&
client_secret=demo_secret
```

**Response (200 OK):**
```json
{
  "access_token": "550e8400-e29b-41d4-a716-446655440000",
  "token_type": "Bearer",
  "expires_in": 3600,
  "scope": "read write"
}
```

**Error Response (400 Bad Request):**
```json
{
  "error": "invalid_grant",
  "error_description": "Authorization code has expired"
}
```

**Security:**
- Authorization code is single-use (deleted after exchange)
- Authorization code expires in 5 minutes
- Client credentials are validated
- Redirect URI must match original request

---

### `GET /oauth/userinfo`
**User info endpoint - protected resource**

Returns authenticated user information. Requires valid access token.

**Request:**
```bash
GET /oauth/userinfo
Authorization: Bearer 550e8400-e29b-41d4-a716-446655440000
```

**Response (200 OK):**
```json
{
  "user_id": "user_001",
  "username": "demo",
  "email": "demo@example.com",
  "name": "Demo User"
}
```

**Error Response (401 Unauthorized):**
```json
{
  "error": "invalid_token",
  "error_description": "Access token is invalid or expired"
}
```

---

## Database Schema

### Tables

#### `clients`
Registered OAuth2 clients (applications).

```sql
CREATE TABLE clients (
    client_id VARCHAR(255) PRIMARY KEY,
    client_secret VARCHAR(255) NOT NULL,
    redirect_uri TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

**Demo Client:**
- `client_id`: demo_client
- `client_secret`: demo_secret
- `redirect_uri`: http://127.0.0.1:8081/callback

---

#### `users`
Demo users for authentication.

```sql
CREATE TABLE users (
    user_id VARCHAR(255) PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

**Demo User:**
- Username: `demo`
- Password: `demo123`
- Password hash: Argon2id

---

#### `authorization_codes`
Short-lived authorization codes (5 minutes TTL).

```sql
CREATE TABLE authorization_codes (
    code VARCHAR(255) PRIMARY KEY,
    client_id VARCHAR(255) NOT NULL REFERENCES clients(client_id),
    user_id VARCHAR(255) NOT NULL REFERENCES users(user_id),
    redirect_uri TEXT NOT NULL,
    scope TEXT,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

**Security:**
- Single-use (deleted after token exchange)
- 5 minute expiration
- Bound to specific redirect_uri

---

#### `access_tokens`
Access tokens for API access (1 hour TTL).

```sql
CREATE TABLE access_tokens (
    token VARCHAR(255) PRIMARY KEY,
    client_id VARCHAR(255) NOT NULL REFERENCES clients(client_id),
    user_id VARCHAR(255) NOT NULL REFERENCES users(user_id),
    scope TEXT,
    expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

**Format:** UUID v4 (opaque token)  
**Alternative:** Could issue JWTs instead (see jwt-service integration)

---

## Configuration

### Environment Variables

```bash
# Server configuration
OAUTH2_SERVER_HOST=127.0.0.1
OAUTH2_SERVER_PORT=8082

# Database configuration
DATABASE_URL=postgres://postgres:postgres@localhost:5432/oauth2_server

# Security (optional, for future enhancements)
# SESSION_SECRET=your-session-secret-key
```

---

## Running the Service

### Prerequisites

```bash
# Start infrastructure
docker compose up -d

# Verify PostgreSQL is running
docker compose ps
```

### Database Setup

```bash
# Run migrations (creates tables + seeds demo data)
cd oauth2-server
sqlx migrate run
```

**What gets created:**
- `clients` table with demo_client
- `users` table with demo user (username: demo, password: demo123)
- `authorization_codes` and `access_tokens` tables (empty)

---

### Start Server

```bash
# From workspace root
cargo run -p oauth2-server

# Or from oauth2-server directory
cd oauth2-server
cargo run
```

Server starts on `http://127.0.0.1:8082`

---

### Test the Flow

**Option 1: Use oauth2-client (recommended)**

```bash
# Terminal 1: oauth2-server
cargo run -p oauth2-server

# Terminal 2: oauth2-client
cargo run -p oauth2-client

# Browser: Complete the flow
open http://localhost:8081
```

**Option 2: Manual browser test**

```bash
# 1. Start oauth2-server
cargo run -p oauth2-server

# 2. Initiate authorization (paste in browser)
open "http://localhost:8082/oauth/authorize?\
client_id=demo_client&\
redirect_uri=http://localhost:8081/callback&\
response_type=code&\
state=test123"

# 3. Approve consent (demo credentials if needed)
#    Username: demo
#    Password: demo123

# 4. You'll be redirected with a code parameter
#    Copy the code from the URL

# 5. Exchange code for token
CODE="paste-code-here"

curl -X POST http://localhost:8082/oauth/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=authorization_code" \
  -d "code=$CODE" \
  -d "redirect_uri=http://localhost:8081/callback" \
  -d "client_id=demo_client" \
  -d "client_secret=demo_secret"

# 6. Use access token
TOKEN="paste-token-here"

curl http://localhost:8082/oauth/userinfo \
  -H "Authorization: Bearer $TOKEN"
```

---

## Security Design

### Authorization Code Security

**Properties:**
- Single-use (prevents replay attacks)
- Short-lived (5 minutes)
- Bound to specific redirect_uri
- Bound to specific client_id

**Attack Mitigation:**
- Code interception → Mitigated by HTTPS (production)
- Code replay → Mitigated by single-use deletion
- Client impersonation → Mitigated by client_secret validation

---

### Client Credentials

**Current Implementation:**
- Client secret stored in database (plaintext)
- Secret transmitted in POST body

**Production Recommendations:**
- Hash client secrets (like passwords)
- Use `Authorization: Basic` header
- Rotate client credentials periodically
- Support public clients with PKCE (no secret)

---

### Password Security

**Current Implementation:**
- Argon2id hashing (recommended by OWASP)
- Salted per-user
- Memory-hard (resistant to GPU attacks)

**Demo Credentials:**
```
Username: demo
Password: demo123
Hash: $argon2id$v=19$m=19456,t=2,p=1$...(truncated)
```

---

### Access Token Security

**Current Format:** UUID v4 (opaque token)

**Security Properties:**
- Unpredictable (cryptographically random)
- Requires database lookup for validation
- Can be revoked instantly

**Alternative:** JWT tokens (see jwt-service integration)

---

## OAuth2 Grant Types

### Currently Implemented

✅ **Authorization Code** - Three-legged OAuth for web apps

### Future Enhancements

- [ ] **Client Credentials** - Machine-to-machine auth
- [ ] **Refresh Token** - Long-lived sessions
- [ ] **PKCE** - Enhanced security for public clients (mobile/SPA)
- [ ] **Device Flow** - For input-constrained devices

---

## Integration with jwt-service

**Scenario:** oauth2-server issues JWTs instead of opaque tokens

**Benefits:**
- Stateless token validation (no database lookup)
- Embedded claims (user_id, scopes, expiration)
- Distributed validation (microservices)

**Implementation:**
```rust
// In token endpoint handler
let claims = Claims::new(user_id, email, 3600);
let access_token = jwt_service::generate_token(&claims, &config.jwt_secret)?;

// Return JWT instead of UUID
TokenResponse {
    access_token,  // JWT format
    token_type: "Bearer",
    expires_in: 3600,
}
```

---

## Implementation Notes

### EMBP Architecture
- Follows Explicit Module Boundary Pattern
- `src/handlers/mod.rs` defines public API
- Sibling imports via `super::`
- Clean separation of concerns

### Error Handling
- Uses `anyhow::Result` for internal errors
- OAuth2-compliant error responses (RFC 6749 Section 5.2)
- Detailed logging for debugging

### Database Access
- Uses `sqlx` for compile-time verified queries
- Connection pooling for performance
- Parameterized queries (SQL injection prevention)

### Manual Request Body Deserialization
- Captures raw request body for debugging
- Logs request before parsing (helps with OAuth2 debugging)
- Provides better error messages

---

## OAuth2 Error Codes

Following RFC 6749 Section 5.2:

| Error Code | Description | HTTP Status |
|------------|-------------|-------------|
| `invalid_request` | Missing required parameter | 400 |
| `invalid_client` | Client authentication failed | 401 |
| `invalid_grant` | Invalid/expired authorization code | 400 |
| `unauthorized_client` | Client not authorized for grant type | 400 |
| `unsupported_grant_type` | Grant type not supported | 400 |
| `invalid_scope` | Requested scope is invalid | 400 |

**Example Error Response:**
```json
{
  "error": "invalid_grant",
  "error_description": "Authorization code has expired"
}
```

---

## Testing

### Unit Tests

```bash
cargo test --package oauth2-server
```

### Integration Tests

See "Test the Flow" section above for end-to-end testing.

### Database Reset

```bash
# Drop and recreate database
docker compose down -v
docker compose up -d
sqlx migrate run
```

---

## Comparison with jwt-service

| Feature | oauth2-server | jwt-service |
|---------|---------------|-------------|
| **Purpose** | Authorization delegation | Token authentication |
| **Protocol** | OAuth2 (RFC 6749) | JWT (RFC 7519) |
| **Token Format** | Opaque UUID | Signed JWT |
| **Validation** | Database lookup | Signature verification |
| **Revocation** | Delete from database | Blacklist (Redis) |
| **Stateless** | No | Yes (except revocation) |
| **Use Case** | Third-party app auth | API authentication |

**Integration:** oauth2-server could call jwt-service to generate JWTs as access tokens.

---

## Future Enhancements

- [ ] Refresh token support
- [ ] PKCE for public clients
- [ ] Client credentials grant type
- [ ] Scope-based authorization
- [ ] JWT access tokens (integrate jwt-service)
- [ ] Admin UI for client registration
- [ ] Rate limiting on token endpoint
- [ ] Audit logging for security events

---

## References

- [JWT Primer](../docs/jwt-primer.md) - JWT concepts and JWT vs OAuth2 comparison
- [EMBP Pattern](https://github.com/JohnBasrai/architecture-patterns/blob/main/rust/embp.md)
- [RFC 6749](https://datatracker.ietf.org/doc/html/rfc6749) - OAuth 2.0 Authorization Framework
- [RFC 6750](https://datatracker.ietf.org/doc/html/rfc6750) - Bearer Token Usage
- [RFC 7636](https://datatracker.ietf.org/doc/html/rfc7636) - PKCE
- [sqlx docs](https://docs.rs/sqlx/) - Rust database toolkit

---

## License

MIT
