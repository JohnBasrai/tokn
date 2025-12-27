// jwt-service/src/handlers/validate.rs

//! Token validation endpoint
//!
//! Handles POST /auth/validate - validates JWT tokens and returns claims

use crate::{token::validate_token, Claims, Config};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// ---

/// Request payload for token validation.
///
/// Client sends the JWT token to be validated.
#[derive(Debug, Deserialize)]
pub struct ValidateRequest {
    // ---
    /// The JWT token to validate
    pub token: String,
}

// ---

/// Successful validation response.
///
/// Contains the decoded claims if the token is valid.
#[derive(Debug, Serialize)]
pub struct ValidateResponse {
    // ---
    /// Indicates the token is valid
    valid: bool,

    /// Decoded claims from the token
    claims: Claims,
}

// ---

/// Error response for invalid tokens.
#[derive(Debug, Serialize)]
pub struct ValidateErrorResponse {
    // ---
    /// Indicates the token is invalid
    valid: bool,

    /// Error message describing why validation failed
    error: String,
}

// ---

/// Validate a JWT token.
///
/// This endpoint verifies the token signature and checks expiration.
///
/// # Request
///
/// ```json
/// {
///   "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
/// }
/// ```
///
/// # Response (200 OK) - Valid Token
///
/// ```json
/// {
///   "valid": true,
///   "claims": {
///     "sub": "user_12345",
///     "email": "john@example.com",
///     "exp": 1703001234,
///     "iat": 1703000334,
///     "jti": "f47ac10b-58cc-4372-a567-0e02b2c3d479"
///   }
/// }
/// ```
///
/// # Response (401 Unauthorized) - Invalid Token
///
/// ```json
/// {
///   "valid": false,
///   "error": "Token has expired"
/// }
/// ```
///
/// # Security
///
/// - Verifies HMAC-SHA256 signature
/// - Checks token expiration (`exp` claim)
/// - Rejects tokens with invalid signatures
/// - Does NOT check revocation list (implement separately)
///
/// # Validation Failures
///
/// Returns 401 Unauthorized if:
/// - Token signature is invalid (tampered or wrong secret)
/// - Token has expired
/// - Token format is malformed
/// - Algorithm is not HS256
///
/// # TODO
///
/// - Add revocation check (query Redis blacklist)
/// - Add rate limiting to prevent validation abuse
/// - Consider caching valid tokens briefly
pub async fn validate_token_handler(
    State(config): State<Arc<Config>>,
    Json(req): Json<ValidateRequest>,
) -> impl IntoResponse {
    // ---
    // Validate the token
    match validate_token(&req.token, &config.jwt.secret) {
        Ok(claims) => {
            // Token is valid
            let response = ValidateResponse {
                valid: true,
                claims,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            // Token is invalid
            tracing::debug!("Token validation failed: {}", e);

            let error_response = ValidateErrorResponse {
                valid: false,
                error: e.to_string(),
            };
            (StatusCode::UNAUTHORIZED, Json(error_response)).into_response()
        }
    }
}
