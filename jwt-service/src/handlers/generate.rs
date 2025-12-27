// jwt-service/src/handlers/generate.rs

//! Token generation endpoint
//!
//! Handles POST /auth/token - generates JWT access tokens

use crate::{token::generate_token, Claims, Config};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// ---

/// Request payload for token generation.
///
/// Client sends this to request a new JWT token.
#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    // ---
    /// User ID to encode in the token
    pub user_id: String,

    /// User email to encode in the token
    pub email: String,
}

// ---

/// Response containing the generated JWT token.
///
/// Follows OAuth2 token response format (RFC 6749 §5.1).
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    // ---
    /// The JWT access token
    access_token: String,

    /// Token type (always "Bearer" for JWTs)
    token_type: String,

    /// Token expiry in seconds
    expires_in: i64,
}

// ---

/// Generate a new JWT access token.
///
/// This endpoint creates a signed JWT containing user claims and returns it to the client.
///
/// # Request
///
/// ```json
/// {
///   "user_id": "user_12345",
///   "email": "john@example.com"
/// }
/// ```
///
/// # Response (200 OK)
///
/// ```json
/// {
///   "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
///   "token_type": "Bearer",
///   "expires_in": 900
/// }
/// ```
///
/// # Security
///
/// - Tokens are signed with HS256 using the JWT_SECRET
/// - Token expiry is configurable (default: 15 minutes)
/// - Each token has a unique `jti` for revocation tracking
///
/// # TODO
///
/// - Add rate limiting to prevent token generation abuse
/// - Add user authentication before issuing tokens (currently trusts request)
/// - Consider adding refresh token in response
///
/// # Errors
///
/// Returns 500 Internal Server Error if token generation fails.
pub async fn generate_token_handler(
    State(config): State<Arc<Config>>,
    Json(req): Json<TokenRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // ---
    // Create claims with configured expiry time
    let claims = Claims::new(
        req.user_id,
        req.email,
        config.jwt.access_token_expiry_seconds,
    );

    // Generate signed JWT
    let token = generate_token(&claims, &config.jwt.secret).map_err(|e| {
        tracing::error!("Token generation failed: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to generate token".to_string(),
        )
    })?;

    // Build response
    let response = TokenResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
        expires_in: config.jwt.access_token_expiry_seconds,
    };

    Ok((StatusCode::OK, Json(response)))
}
