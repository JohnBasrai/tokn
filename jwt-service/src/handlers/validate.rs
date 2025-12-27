// jwt-service/src/handlers/validate.rs

//! Token validation endpoint
//!
//! Handles POST /auth/validate - validates JWT tokens and returns claims

use crate::{is_token_revoked, token::validate_token, AppState, Claims};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};

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
/// This endpoint verifies the token signature, checks expiration, and checks
/// if the token has been revoked.
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
/// - **Checks revocation blacklist** (new!)
/// - Rejects tokens with invalid signatures
///
/// # Validation Order
///
/// 1. Verify signature (fail fast on tampered tokens)
/// 2. Check expiration (fail fast on expired tokens)
/// 3. **Check revocation blacklist** (only if token is otherwise valid)
///
/// # Validation Failures
///
/// Returns 401 Unauthorized if:
/// - Token signature is invalid (tampered or wrong secret)
/// - Token has expired
/// - Token format is malformed
/// - Algorithm is not HS256
/// - **Token has been revoked** (in blacklist)
///
/// # TODO
///
/// - Add rate limiting to prevent validation abuse
/// - Consider caching valid tokens briefly
pub async fn validate_token_handler(
    State(state): State<AppState>,
    Json(req): Json<ValidateRequest>,
) -> impl IntoResponse {
    // ---
    // Validate the token (signature + expiry)
    let claims = match validate_token(&req.token, &state.config.jwt.secret) {
        Ok(claims) => claims,
        Err(e) => {
            // Token is invalid
            tracing::debug!("Token validation failed: {}", e);

            let error_response = ValidateErrorResponse {
                valid: false,
                error: e.to_string(),
            };
            return (StatusCode::UNAUTHORIZED, Json(error_response)).into_response();
        }
    };

    // Check if token is revoked
    match is_token_revoked(&mut state.redis.clone(), &claims.jti).await {
        Ok(true) => {
            // Token is revoked
            tracing::debug!(
                "Token validation failed: token revoked (jti={})",
                claims.jti
            );

            let error_response = ValidateErrorResponse {
                valid: false,
                error: "Token has been revoked".to_string(),
            };
            return (StatusCode::UNAUTHORIZED, Json(error_response)).into_response();
        }
        Ok(false) => {
            // Token is not revoked, proceed
        }
        Err(e) => {
            // Redis error - fail secure (reject token)
            tracing::error!("Failed to check token revocation status: {}", e);

            let error_response = ValidateErrorResponse {
                valid: false,
                error: "Failed to verify token status".to_string(),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response();
        }
    }

    // Token is valid and not revoked
    let response = ValidateResponse {
        valid: true,
        claims,
    };
    (StatusCode::OK, Json(response)).into_response()
}
