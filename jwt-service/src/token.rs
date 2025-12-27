// jwt-service/src/token.rs

//! JWT token generation and encoding
//!
//! Provides functions to generate signed JWT tokens using HS256 algorithm.

use crate::claims::Claims;
use anyhow::{Context, Result};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};

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
