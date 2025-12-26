# Contributing to Tokn (rust-auth-infrastructure)

Thanks for considering contributing!

**Before submitting a pull request:**

- Ensure all tests pass (`SQLX_OFFLINE=true cargo test`)
- Format your code (`cargo fmt`)
- Run clippy checks (`SQLX_OFFLINE=true cargo clippy`)
- If your change affects behavior, please update `CHANGELOG.md` under the [Unreleased] section
- Keep commits focused and descriptive

**Note:** See [docs/sqlx-offline-mode-howto.md](docs/sqlx-offline-mode-howto.md) for information about SQLx offline mode.

We follow [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) and [Semantic Versioning](https://semver.org/).

## Code Formatting

This project uses `rustfmt` for consistent code formatting. All code should be formatted before committing.

### Quick Commands
```bash
# Format all code
cargo fmt

# Check if code is formatted (used by CI)
cargo fmt --check

# Run clippy linter
SQLX_OFFLINE=true cargo clippy --all-targets --all-features --no-deps -- -D warnings

# Run the complete test suite
./scripts/test-all.sh

# Run tests for specific crate
SQLX_OFFLINE=true cargo test -p oauth2-client
SQLX_OFFLINE=true cargo test -p oauth2-server

# Run full CI/CD run locally to increase the chances that it will run on GitHub.
scripts/ci-local.sh
```

### Visual Separators

Since `rustfmt` removes blank lines at the start of impl blocks, function bodies, and module blocks, we use comment separators `// ---` for visual clarity:

```rust
// Module blocks
mod helpers {
    // ---
    use super::*;

    pub fn some_function() {
        // ---
        // function body
    }
}

// Struct definitions
pub struct UserInfo {
    // ---
    sub: String,
    username: String,
}

// Impl blocks
impl TokenService for JwtTokenService {
    // ---
    fn generate_token(&self, user_id: &str) -> Result<String> {
        // ---
        // implementation
    }
}

// Regular functions
pub fn validate_authorization_code() {
    // ---
    let code = AuthorizationCode::new(code_string);
    // ...
}

// Struct literals (construction) - NO separator
let config = Config {
    server,
    redis,
    oauth2,
};

// Test modules
#[cfg(test)]
mod tests {
    // ---
    use super::*;

    #[test]
    fn test_token_exchange() {
        // ---
        // test body
    }
}
```

**Style Guidelines:**
1) Use `// ---` for visual separation in at a minimum **module blocks**, **impl blocks**, **struct definitions**, and **function bodies**
2) Place separators after the opening brace and before the first meaningful line
3) Between meaningful steps of logic processing (e.g., separating validation, database operations, and response formatting - see `token_handler` in `oauth2-server/src/handlers/token.rs`)
4) For modules: place separator after `mod name {` and before imports/content
5) For impl blocks: place separator after `impl ... {` and before the first method
6) For struct definitions: place separator after `struct Name {` and before field declarations
7) For functions: place separator after function signature and before the main logic
8) Do NOT use separators inside struct literals (during construction)
9) Keep separators consistent across the codebase

**Note:** This project uses rustfmt's default configuration. The `// ---` separator pattern is a formatting convention to work around rustfmt's blank line removal in stable Rust.

## Documentation and Doc Comments

This project follows a **production-grade documentation standard** for Rust code, with special attention to OAuth2/OIDC authentication flows.

### Required Doc Comments

Use Rust doc comments (`///`) for:

- Public structs and enums (especially OAuth2-related types like `AuthorizeQuery`, `TokenRequest`)
- Public functions (especially handlers and service methods)
- Public modules that define architectural boundaries
- Security-critical code (token validation, authorization flows)
- Macros that encode non-obvious behavior or policy decisions

Doc comments should describe **intent, guarantees, and failure semantics** —
not restate what the code obviously does.

### OAuth2/Security-Specific Documentation

For authentication and authorization code, doc comments should explicitly describe:

- **Security implications** - What security guarantees does this provide?
- **OAuth2 flow stage** - Which part of the authorization code flow is this?
- **Failure modes** - What happens when tokens expire, validation fails, etc.?
- **RFC compliance** - Reference relevant OAuth2/OIDC RFCs when applicable

Example:
```rust
/// Exchanges an authorization code for an access token.
///
/// This implements the token exchange step of the OAuth2 authorization code flow (RFC 6749 §4.1.3).
///
/// # Security
///
/// - Validates the authorization code exists and hasn't been used
/// - Verifies the redirect URI matches the one used during authorization
/// - Invalidates the code after successful exchange (one-time use)
///
/// # Errors
///
/// Returns an error if:
/// - The authorization code is invalid or expired
/// - The redirect URI doesn't match
/// - The code has already been exchanged
pub async fn exchange_code(code: &str, redirect_uri: &str) -> Result<AccessToken> {
    // ---
    // implementation
}
```

### Optional (Encouraged) Doc Comments

Doc comments or short block comments are encouraged for:

- Internal functions with security or operational implications
- Token generation and validation logic
- Configuration parsing and validation
- Code that enforces OAuth2 protocol requirements
- Startup and initialization logic

### Not Required

Doc comments are not required for:

- Trivial helpers
- Simple getters or pass-through functions
- Test code (assert messages should be sufficient)
- Obvious glue code

### General Guidance

- Prefer documenting *why* over *how*
- Be explicit about failure behavior
- Keep comments accurate and up to date
- Avoid over-documenting trivial code
- For OAuth2 flows, reference the relevant RFC section when applicable

Well-written doc comments are considered part of the code's correctness, especially for security-critical authentication code.

## Architecture Guidelines

This project uses the [Explicit Module Boundary Pattern (EMBP)](https://github.com/JohnBasrai/architecture-patterns/blob/main/rust/embp.md) for module organization. Please review the EMBP documentation before making structural changes.

### Key EMBP Principles

- Each module's public API is defined in its `mod.rs` gateway file
- Sibling modules import from each other using `super::`
- External modules import through `crate::module::`
- Never bypass module gateways with deep imports

## Testing OAuth2 Flows

When testing authentication flows:

- Use Docker Compose for integration tests (`docker-compose up -d`)
- Test both success and failure paths
- Verify token expiration and refresh flows
- Test error handling for invalid/expired tokens
- Include tests for security edge cases (CSRF, token replay, etc.)
