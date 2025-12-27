// jwt-service/src/handlers/generate.rs

//! Token generation endpoint
//!
//! Handles POST /auth/token - generates JWT access tokens and refresh tokens

use crate::{generate_refresh_token, token::generate_token, AppState, Claims};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};

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

/// Response containing the generated JWT and refresh tokens.
///
/// Follows OAuth2 token response format (RFC 6749 §5.1) with refresh token extension.
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    // ---
    /// The JWT access token
    access_token: String,

    /// Token type (always "Bearer" for JWTs)
    token_type: String,

    /// Access token expiry in seconds
    expires_in: i64,

    /// Refresh token for obtaining new access tokens
    refresh_token: String,
}

// ---

/// Generate new JWT access token and refresh token.
///
/// This endpoint creates a signed JWT access token and a refresh token stored in Redis.
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
///   "expires_in": 900,
///   "refresh_token": "f47ac10b-58cc-4372-a567-0e02b2c3d479"
/// }
/// ```
///
/// # Security
///
/// - Access tokens are signed with HS256 using JWT_SECRET
/// - Access token expiry is configurable (default: 15 minutes)
/// - Each access token has a unique `jti` for revocation tracking
/// - Refresh tokens are random UUIDs stored in Redis
/// - Refresh tokens expire after configured duration (default: 7 days)
/// - Refresh tokens are single-use (deleted on refresh)
///
/// # Token Workflow
///
/// 1. Client uses access token for API requests
/// 2. Access token expires after 15 minutes
/// 3. Client uses refresh token to get new access token
/// 4. Refresh token is consumed and new one issued (rotation)
/// 5. After 7 days, refresh token expires and user must re-authenticate
///
/// # TODO
///
/// - Add rate limiting to prevent token generation abuse
/// - Add user authentication before issuing tokens (currently trusts request)
///
/// # Errors
///
/// Returns 500 Internal Server Error if token generation or Redis storage fails.
pub async fn generate_token_handler(
    State(state): State<AppState>,
    Json(req): Json<TokenRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // ---
    // Create claims with configured expiry time
    let claims = Claims::new(
        req.user_id.clone(),
        req.email.clone(),
        state.config.jwt.access_token_expiry_seconds,
    );

    // Generate signed JWT access token
    let access_token = generate_token(&claims, &state.config.jwt.secret).map_err(|e| {
        tracing::error!("Token generation failed: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to generate token".to_string(),
        )
    })?;

    // Generate and store refresh token
    let refresh_token = generate_refresh_token(
        &mut state.redis.clone(),
        &req.user_id,
        &req.email,
        state.config.jwt.refresh_token_expiry_seconds,
    )
    .await
    .map_err(|e| {
        tracing::error!("Refresh token generation failed: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to generate refresh token".to_string(),
        )
    })?;

    // Build response
    let response = TokenResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: state.config.jwt.access_token_expiry_seconds,
        refresh_token,
    };

    Ok((StatusCode::OK, Json(response)))
}
