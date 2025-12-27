// jwt-service/src/claims.rs

//! JWT claims structures
//!
//! Defines the payload that will be encoded in JWT tokens.

use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---

/// JWT Claims following RFC 7519 standard claims.
///
/// # Standard Claims
///
/// - `sub` (subject) - User identifier
/// - `exp` (expiration) - When the token expires (Unix timestamp)
/// - `iat` (issued at) - When the token was created (Unix timestamp)
/// - `jti` (JWT ID) - Unique token identifier for revocation
///
/// # Custom Claims
///
/// - `email` - User email address (application-specific)
///
/// # Security
///
/// - Never put sensitive data (passwords, credit cards) in claims
/// - Claims are Base64-encoded, NOT encrypted
/// - Anyone can decode and read the claims (signature only proves authenticity)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    // ---
    /// Subject - User ID this token represents
    pub sub: String,

    /// Email - Custom claim for user email
    pub email: String,

    /// Expiration time (Unix timestamp)
    pub exp: usize,

    /// Issued at time (Unix timestamp)
    pub iat: usize,

    /// JWT ID - Unique identifier for this token (used for revocation)
    pub jti: String,
}

// ---

impl Claims {
    // ---
    /// Create new claims with standard expiration time.
    ///
    /// # Arguments
    ///
    /// - `user_id` - Unique identifier for the user
    /// - `email` - User's email address
    /// - `expiry_seconds` - Token expiry duration in seconds (e.g., 900 = 15 minutes)
    ///
    /// # Returns
    ///
    /// Claims with:
    /// - `sub` set to user_id
    /// - `email` set to provided email
    /// - `iat` set to current time
    /// - `exp` set to current time + expiry_seconds
    /// - `jti` set to random UUID v4
    ///
    /// # Example
    ///
    /// ```no_run
    /// use jwt_service::Claims;
    ///
    /// let claims = Claims::new(
    ///     "user_12345".to_string(),
    ///     "john@example.com".to_string(),
    ///     900  // 15 minutes
    /// );
    /// ```
    pub fn new(user_id: String, email: String, expiry_seconds: i64) -> Self {
        // ---
        let now = Utc::now();
        let exp_time = now + Duration::seconds(expiry_seconds);

        Self {
            sub: user_id,
            email,
            iat: now.timestamp() as usize,
            exp: exp_time.timestamp() as usize,
            jti: Uuid::new_v4().to_string(),
        }
    }
}
