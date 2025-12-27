// jwt-service/src/token.rs

//! JWT token generation and encoding
//!
//! Provides functions to generate and validate signed JWT tokens using HS256 algorithm.

use crate::claims::Claims;
use anyhow::{Context, Result};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

// ---

/// Generate a signed JWT token from claims.
///
/// Uses HS256 (HMAC-SHA256) for signing. The secret key must be at least 256 bits (32 bytes).
///
/// # Arguments
///
/// - `claims` - The claims to encode in the token
/// - `secret` - Secret key for HMAC signing (minimum 32 bytes)
///
/// # Returns
///
/// A signed JWT string in the format: `header.payload.signature`
///
/// # Security
///
/// - Secret must be kept confidential
/// - Secret should be at least 256 bits (32 bytes) for HS256
/// - Rotate secrets periodically
/// - Use environment variables, never hardcode
///
/// # Errors
///
/// Returns error if:
/// - JWT encoding fails
/// - Secret is invalid
///
/// # Example
///
/// ```no_run
/// use jwt_service::{Claims, generate_token};
///
/// let claims = Claims::new(
///     "user_123".to_string(),
///     "user@example.com".to_string(),
///     900
/// );
///
/// let secret = "your-secret-key-at-least-32-characters";
/// let token = generate_token(&claims, secret)?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn generate_token(claims: &Claims, secret: &str) -> Result<String> {
    // ---
    let header = Header::new(Algorithm::HS256);
    let encoding_key = EncodingKey::from_secret(secret.as_bytes());

    encode(&header, claims, &encoding_key).context("Failed to encode JWT token")
}

// ---

/// Validate and decode a JWT token.
///
/// Verifies the token signature and checks expiration. Returns the decoded claims if valid.
///
/// # Arguments
///
/// - `token` - The JWT string to validate
/// - `secret` - Secret key used to sign the token (must match generation secret)
///
/// # Returns
///
/// The decoded `Claims` if the token is valid.
///
/// # Security
///
/// Validates:
/// - **Signature** - Token hasn't been tampered with (HS256 verification)
/// - **Expiration** - Token hasn't expired (checks `exp` claim)
/// - **Algorithm** - Only HS256 is accepted (prevents algorithm confusion attacks)
///
/// Does NOT validate:
/// - Token revocation (check blacklist separately)
/// - Issuer or audience (not enforced by default)
///
/// # Errors
///
/// Returns error if:
/// - Signature is invalid (token was tampered with or wrong secret)
/// - Token has expired (`exp` claim is in the past)
/// - Token format is invalid
/// - Algorithm is not HS256
///
/// # Example
///
/// ```no_run
/// use jwt_service::validate_token;
///
/// let secret = "your-secret-key-at-least-32-characters";
/// let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...";
///
/// match validate_token(token, secret) {
///     Ok(claims) => println!("Valid token for user: {}", claims.sub),
///     Err(e) => println!("Invalid token: {}", e),
/// }
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn validate_token(token: &str, secret: &str) -> Result<Claims> {
    // ---
    let decoding_key = DecodingKey::from_secret(secret.as_bytes());

    // Configure validation rules
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true; // Check expiration (default, but explicit)

    // Decode and validate token
    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .context("Failed to validate JWT token")?;

    Ok(token_data.claims)
}
