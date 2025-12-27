# JWT Primer: Understanding JSON Web Tokens

**Target Audience:** Developers new to JWT or reviewing authentication patterns  
**Last Updated:** December 2024  
**Scope:** Theory and concepts (implementation-agnostic)

---

## What is a JWT?

A **JSON Web Token (JWT)** is a compact, URL-safe means of representing claims between two parties. JWTs are self-contained: they carry all the information needed to verify their authenticity without requiring a database lookup.

**Key Properties:**
- **Stateless** - No server-side session storage required
- **Self-contained** - All user info is in the token itself
- **Cryptographically signed** - Cannot be tampered with
- **Compact** - URL-safe, can be sent in HTTP headers

---

## JWT Structure

A JWT consists of three Base64URL-encoded parts separated by dots (`.`):

```
HEADER.PAYLOAD.SIGNATURE
```

### Example JWT:
```
eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c
```

### 1. Header

Describes the token type and signing algorithm:

```json
{
  "alg": "HS256",
  "typ": "JWT"
}
```

**Common Algorithms:**
- `HS256` - HMAC SHA-256 (symmetric, shared secret)
- `RS256` - RSA SHA-256 (asymmetric, public/private key)
- `ES256` - ECDSA SHA-256 (asymmetric, elliptic curve)

### 2. Payload (Claims)

Contains the data being transmitted. Claims can be:

**Registered Claims** (standard, defined by RFC 7519):
- `iss` (issuer) - Who created the token
- `sub` (subject) - Who the token is about (usually user ID)
- `aud` (audience) - Who the token is intended for
- `exp` (expiration time) - When the token expires (Unix timestamp)
- `nbf` (not before) - Token not valid before this time
- `iat` (issued at) - When the token was created
- `jti` (JWT ID) - Unique identifier for the token

**Public Claims** (custom, but should be collision-resistant):
- Use namespaced names like `https://example.com/user_role`

**Private Claims** (custom, agreed upon by parties):
- `user_id`, `email`, `role`, etc.

```json
{
  "sub": "user_12345",
  "name": "John Doe",
  "email": "john@example.com",
  "role": "admin",
  "iat": 1516239022,
  "exp": 1516242622
}
```

**⚠️ Security Note:** Never put sensitive data (passwords, credit cards) in the payload—it's only Base64-encoded, not encrypted!

### 3. Signature

Ensures the token hasn't been tampered with. Created by:

1. Take the encoded header and payload
2. Concatenate with a dot: `encodedHeader.encodedPayload`
3. Sign with the secret key and algorithm specified in header

**Example (HS256):**
```
HMACSHA256(
  base64UrlEncode(header) + "." + base64UrlEncode(payload),
  secret
)
```

---

## How JWTs Work

### **Token Generation:**
1. Server creates claims (user info, expiration, etc.)
2. Server signs claims with secret key
3. Server returns JWT to client

### **Token Validation:**
1. Client sends JWT in request (usually in `Authorization: Bearer <token>` header)
2. Server decodes the JWT
3. Server verifies signature using secret key
4. Server checks expiration (`exp` claim)
5. If valid, server trusts the claims and processes request

---

## JWT vs. Session Tokens

| Aspect | Session Tokens | JWTs |
|--------|---------------|------|
| **Storage** | Server stores session state (Redis, database) | Client stores token, server is stateless |
| **Database Lookup** | Required on every request | Not required (unless checking revocation) |
| **Scalability** | Harder (session replication across servers) | Easier (any server can validate) |
| **Revocation** | Easy (delete session from store) | Hard (need blacklist or short expiry) |
| **Size** | Small (just session ID) | Larger (contains all claims) |
| **Best For** | Traditional web apps with server-side state | APIs, microservices, distributed systems |

---

## Common JWT Use Cases

### 1. **API Authentication**
- Mobile app → API server
- SPA (Single Page App) → Backend API
- Microservice-to-microservice communication

### 2. **Single Sign-On (SSO)**
- User logs in once, gets JWT
- JWT works across multiple services

### 3. **Stateless Authorization**
- JWT contains user roles/permissions
- Each service can check authorization without database call

---

## Security Considerations

### ✅ **Do:**
- Use strong secrets (256+ bits for HS256)
- Set short expiration times (`exp` claim)
- Use HTTPS to prevent token interception
- Validate `exp`, `iat`, and `nbf` claims
- Implement refresh token rotation
- Store tokens securely (httpOnly cookies, not localStorage for web)

### ❌ **Don't:**
- Put sensitive data in JWT payload (it's not encrypted)
- Use weak signing algorithms (avoid `none`, `HS256` with weak secrets)
- Share JWTs across untrusted domains
- Ignore token expiration
- Store JWTs in localStorage (XSS vulnerability for web apps)

---

## Token Revocation Strategies

**Problem:** JWTs are stateless, so you can't "delete" them like sessions.

**Solutions:**

### 1. **Short Expiration + Refresh Tokens**
- Access token: 15 minutes expiry
- Refresh token: 7 days expiry, stored server-side
- Client exchanges refresh token for new access token

### 2. **Token Blacklisting**
- Store revoked tokens in Redis with TTL = token expiry
- Check blacklist during validation (adds latency)

### 3. **Version Claims**
- Add `token_version` to user record in database
- Include `token_version` in JWT
- Invalidate all tokens by incrementing user's `token_version`

---

## Access Tokens vs. Refresh Tokens

| Token Type | Purpose | Expiry | Storage |
|------------|---------|--------|---------|
| **Access Token** | Access protected resources | Short (15 min) | Memory, short-lived |
| **Refresh Token** | Get new access tokens | Long (7 days) | Secure storage (httpOnly cookie, database) |

**Flow:**
1. User logs in → Gets access token + refresh token
2. Client uses access token for API requests
3. Access token expires → Client uses refresh token to get new access token
4. Refresh token expires → User must log in again

**Security Benefit:** If access token is stolen, it's only valid for 15 minutes.

---

## JWT vs. OAuth2

**They're complementary, not competing:**

- **OAuth2** is an *authorization framework* (how to delegate access)
- **JWT** is a *token format* (how to encode claims)

**OAuth2 can use JWTs:**
- OAuth2 access tokens can be JWTs (contains scopes, user ID)
- Or OAuth2 can use opaque tokens (just random strings, need database lookup)

**Example:**
1. User authorizes app via OAuth2 authorization code flow
2. OAuth2 server issues an **access token** (could be a JWT)
3. Client uses JWT access token to call protected APIs

---

## When to Use JWTs

### ✅ **Use JWTs when:**
- Building stateless APIs (no session storage)
- Microservices architecture (each service validates independently)
- Mobile or SPA clients (can store tokens locally)
- Cross-domain authentication (SSO)
- You need to embed user metadata in the token

### ❌ **Don't use JWTs when:**
- You need instant token revocation (use sessions instead)
- Tokens will be very large (100+ claims)
- Web app where httpOnly cookies + sessions work fine
- Token theft is a high risk (short-lived sessions are safer)

---

## RFC Standards

- **RFC 7519:** JSON Web Token (JWT)
- **RFC 7515:** JSON Web Signature (JWS)
- **RFC 7516:** JSON Web Encryption (JWE)
- **RFC 7517:** JSON Web Key (JWK)

---

## Additional Resources

- [jwt.io](https://jwt.io) - Decode and debug JWTs
- [OWASP JWT Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/JSON_Web_Token_for_Java_Cheat_Sheet.html)
- [RFC 7519 (JWT Spec)](https://datatracker.ietf.org/doc/html/rfc7519)

---

**Next Steps:**
- Read `jwt-service/README.md` for implementation details in Rust
- Explore `jwt-service/src/` for code examples
