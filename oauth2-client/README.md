# oauth2-client

**OAuth2 client demonstration** implementing the authorization code flow with PKCE using `oauth2-server` as the authorization server.

**Port:** 8081

---

## Overview

This service demonstrates a web application acting as an **OAuth2 client** (relying party). It showcases:

- **Authorization Code Flow** - Redirect user to authorization server for consent
- **Token Exchange** - Exchange authorization code for access token
- **Protected Resources** - Access user info with bearer token
- **State Parameter** - CSRF protection during OAuth2 flow
- **PKCE** - Future enhancement for mobile/SPA security

---

## Architecture

### OAuth2 Roles

In the OAuth2 flow, this service plays the **Client** role:

- **Resource Owner** - End user (you, in the browser)
- **Client** - **This application** (oauth2-client)
- **Authorization Server** - oauth2-server (port 8082)
- **Resource Server** - oauth2-server's `/oauth/userinfo` endpoint

### Authorization Code Flow Sequence

```
1. User clicks "Login" → Redirects to /oauth/authorize
   ├─→ oauth2-server shows consent page
   └─→ User approves access

2. oauth2-server redirects back to /callback with code
   ├─→ Client exchanges code for access_token
   └─→ Client stores token (in-memory for demo)

3. Client fetches user info from /oauth/userinfo
   └─→ Displays profile page with user data
```

---

## Endpoints

### `GET /`
**Home page**

Shows login button to start OAuth2 flow.

**Response (HTML):**
```html
<h1>OAuth2 Client Demo</h1>
<a href="/login">Login with OAuth2</a>
```

---

### `GET /login`
**Initiate OAuth2 flow**

Redirects user to `oauth2-server` authorization endpoint with:
- `client_id` - Identifies this application
- `redirect_uri` - Where to send user after authorization
- `state` - Random value for CSRF protection
- `response_type=code` - Request authorization code

**Redirects to:**
```
http://localhost:8082/oauth/authorize?
  client_id=demo-client
  &redirect_uri=http://localhost:8081/callback
  &state=random-csrf-token
  &response_type=code
```

---

### `GET /callback`
**Handle OAuth2 callback**

Receives authorization code from oauth2-server, exchanges it for access token.

**Query Parameters:**
- `code` - Authorization code (short-lived, single-use)
- `state` - CSRF protection token (must match original)

**Flow:**
1. Validate state parameter (CSRF check)
2. Exchange code for access_token via POST /oauth/token
3. Store access_token (in-memory for demo)
4. Fetch user info from /oauth/userinfo
5. Display profile page

**Redirects to:** `/profile`

---

### `GET /profile`
**Display user profile**

Shows user information fetched from authorization server.

**Response (HTML):**
```html
<h1>Profile</h1>
<p>User ID: user_alice</p>
<p>Email: alice@example.com</p>
<p>Name: Alice Smith</p>
```

**Security:** In production, this would validate the session/token.

---

## Configuration

### Environment Variables

```bash
# oauth2-client configuration
OAUTH2_CLIENT_HOST=127.0.0.1
OAUTH2_CLIENT_PORT=8081

# OAuth2 provider (oauth2-server)
OAUTH2_CLIENT_ID=demo-client
OAUTH2_CLIENT_SECRET=demo-secret
OAUTH2_AUTH_URL=http://localhost:8082/oauth/authorize
OAUTH2_TOKEN_URL=http://localhost:8082/oauth/token
OAUTH2_USERINFO_URL=http://localhost:8082/oauth/userinfo
OAUTH2_REDIRECT_URI=http://localhost:8081/callback
```

**Security Notes:**
- `client_secret` should be stored securely (environment variable, not hardcoded)
- In production, use HTTPS for all URLs
- `redirect_uri` must match exactly what's registered with authorization server

---

## Running the Service

### Prerequisites
- Docker and Docker Compose (for infrastructure)
- oauth2-server must be running on port 8082
- PostgreSQL and Redis must be running

### Start oauth2-server First

```bash
# From workspace root
docker compose up -d

# Start oauth2-server
cargo run -p oauth2-server
```

### Start oauth2-client

```bash
# From workspace root
cargo run -p oauth2-client
```

### Test the Flow

1. Open browser: http://localhost:8081
2. Click "Login with OAuth2"
3. Approve consent on oauth2-server page
4. View your profile at http://localhost:8081/profile

---

## Testing

### Integration Test

```bash
# Full OAuth2 flow test
# 1. Start oauth2-server (terminal 1)
cargo run -p oauth2-server

# 2. Start oauth2-client (terminal 2)
cargo run -p oauth2-client

# 3. Open browser and test flow
open http://localhost:8081
```

### Manual API Testing

```bash
# Step 1: Get authorization code (do this in browser)
open "http://localhost:8082/oauth/authorize?\
client_id=demo-client&\
redirect_uri=http://localhost:8081/callback&\
response_type=code&\
state=test123"

# Step 2: Exchange code for token (copy code from redirect)
CODE="paste-code-from-redirect-here"

curl -X POST http://localhost:8082/oauth/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=authorization_code" \
  -d "code=$CODE" \
  -d "redirect_uri=http://localhost:8081/callback" \
  -d "client_id=demo-client" \
  -d "client_secret=demo-secret"

# Step 3: Fetch user info with access token
TOKEN="paste-token-here"

curl http://localhost:8082/oauth/userinfo \
  -H "Authorization: Bearer $TOKEN"
```

---

## Security Considerations

### State Parameter (CSRF Protection)

The OAuth2 flow is vulnerable to CSRF attacks without the `state` parameter:

**Attack scenario:**
1. Attacker initiates OAuth2 flow and obtains authorization code
2. Attacker tricks victim into visiting: `http://localhost:8081/callback?code=ATTACKERS_CODE`
3. Victim's session is now linked to attacker's account

**Mitigation:**
- Generate random `state` value at `/login`
- Store state in session or cookie
- Verify state matches at `/callback`

**Current implementation:** Basic state validation (in-memory)

---

### Client Secret Protection

**Current implementation:** Client secret is sent in POST body to token endpoint.

**Production recommendations:**
- Use `Authorization: Basic base64(client_id:client_secret)` header
- Never expose client_secret in frontend JavaScript
- For public clients (SPAs, mobile), use PKCE instead of client_secret

---

### Token Storage

**Current implementation:** Tokens stored in-memory (lost on restart).

**Production recommendations:**
- Server-side sessions with httpOnly cookies
- Never store access_token in localStorage (XSS risk)
- Consider refresh tokens for long-lived sessions

---

## OAuth2 vs Session-Based Auth

| Feature | OAuth2 (this demo) | Session (cr8s) |
|---------|-------------------|----------------|
| **Use Case** | Third-party app authorization | First-party authentication |
| **Complexity** | High (delegated auth) | Low (direct login) |
| **User Control** | Granular consent | All-or-nothing |
| **Token Type** | Access token (stateless JWT or opaque) | Session ID (opaque) |
| **Best For** | "Login with Google", API access | Traditional web apps |

**When to use OAuth2:**
- Social login (Google, GitHub, etc.)
- Third-party app needs limited access
- Microservices with separate auth server

**When to use sessions:**
- First-party web application
- Simpler infrastructure requirements
- Don't need delegation features

---

## Implementation Notes

### EMBP Architecture
- Follows Explicit Module Boundary Pattern
- `src/handlers/mod.rs` defines public API
- Sibling imports via `super::`

### Error Handling
- Uses `anyhow::Result` for ergonomic error handling
- Detailed error messages for debugging
- HTML error pages for user-facing errors

### OAuth2 Crate
- Built on `oauth2` crate (https://docs.rs/oauth2/)
- Provides type-safe OAuth2 client
- Handles URL generation and token exchange

---

## Future Enhancements

- [ ] PKCE support (for public clients)
- [ ] Proper session management (Redis)
- [ ] Refresh token flow
- [ ] Scope-based authorization
- [ ] Multiple OAuth2 providers (GitHub, Google)
- [ ] JWT token validation (if oauth2-server issues JWTs)

---

## Comparison with oauth2-server

| Feature | oauth2-client (this) | oauth2-server |
|---------|---------------------|---------------|
| **Role** | Client (relying party) | Authorization server |
| **Endpoints** | `/login`, `/callback`, `/profile` | `/oauth/authorize`, `/oauth/token`, `/oauth/userinfo` |
| **Initiates** | Authorization request | Issues tokens |
| **Stores** | Access tokens (temporary) | User credentials, codes, tokens |

**Relationship:** oauth2-client depends on oauth2-server for authorization.

---

## References

- [JWT Primer](../docs/jwt-primer.md) - JWT concepts and JWT vs OAuth2 comparison
- [EMBP Pattern](https://github.com/JohnBasrai/architecture-patterns/blob/main/rust/embp.md)
- [RFC 6749](https://datatracker.ietf.org/doc/html/rfc6749) - OAuth2 specification
- [RFC 7636](https://datatracker.ietf.org/doc/html/rfc7636) - PKCE specification
- [oauth2 crate](https://docs.rs/oauth2/) - Rust OAuth2 client library

---

## License

MIT
