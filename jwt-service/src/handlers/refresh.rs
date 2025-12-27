// jwt-service/src/handlers/refresh.rs

//! Refresh token endpoint
//!
//! Handles POST /auth/refresh - exchanges refresh tokens for new access tokens

use crate::{
    generate_refresh_token, token::generate_token, validate_refresh_token, AppState, Claims,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};

// ---

/// Request payload for token refresh.
///
/// Client sends the refresh token to get a new access token.
#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    // ---
    /// The refresh token to exchange
    pub refresh_token: String,
}

// ---

/// Response containing new access and refresh tokens.
///
/// Returns new tokens after successful refresh.
#[derive(Debug, Serialize)]
pub struct RefreshResponse {
    // ---
    /// New JWT access token
    access_token: String,

    /// Token type (always "Bearer" for JWTs)
    token_type: String,

    /// Access token expiry in seconds
    expires_in: i64,

    /// New refresh token (rotation)
    refresh_token: String,
}

// ---

/// Exchange a refresh token for new access and refresh tokens.
///
/// This endpoint implements **refresh token rotation**: the provided refresh token
/// is consumed (deleted) and a new one is issued along with a new access token.
///
/// # Request
///
/// ```json
/// {
///   "refresh_token": "f47ac10b-58cc-4372-a567-0e02b2c3d479"
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
///   "refresh_token": "a1b2c3d4-e5f6-7890-abcd-ef1234567890"
/// }
/// ```
///
/// # Response (401 Unauthorized) - Invalid/Expired Token
///
/// ```json
/// {
///   "error": "Invalid or expired refresh token"
/// }
/// ```
///
/// # Security - Token Rotation
///
/// This endpoint implements **refresh token rotation** to prevent token replay attacks:
///
/// 1. Validates the refresh token exists in Redis
/// 2. Retrieves the user data
/// 3. **Deletes the old refresh token** (one-time use)
/// 4. Generates new access token
/// 5. Generates and stores new refresh token
///
/// **Why rotation matters:**
/// - If an attacker steals a refresh token, it can only be used once
/// - The next legitimate refresh attempt will fail
/// - User is alerted to revoke all sessions
/// - Prevents long-lived stolen tokens from being reused
///
/// # Errors
///
/// Returns 401 Unauthorized if:
/// - Refresh token doesn't exist in Redis (expired or already used)
/// - Refresh token format is invalid
///
/// Returns 500 Internal Server Error if:
/// - Token generation fails
/// - Redis operations fail
///
/// # Example Flow
///
/// ```text
/// 1. User logs in → Gets access token (15 min) + refresh token (7 days)
/// 2. After 15 min → Access token expires
/// 3. Client sends refresh token to /auth/refresh
/// 4. Old refresh token deleted, new tokens issued
/// 5. Repeat until refresh token expires (7 days)
/// 6. User must re-authenticate
/// ```
pub async fn refresh_token_handler(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> impl IntoResponse {
    // ---
    // Validate and consume refresh token (deletes it from Redis)
    let user_data = match validate_refresh_token(&mut state.redis.clone(), &req.refresh_token).await
    {
        Ok(data) => data,
        Err(e) => {
            tracing::debug!("Refresh token validation failed: {}", e);
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Invalid or expired refresh token"
                })),
            )
                .into_response();
        }
    };

    // Generate new access token
    let claims = Claims::new(
        user_data.user_id.clone(),
        user_data.email.clone(),
        state.config.jwt.access_token_expiry_seconds,
    );

    let access_token = match generate_token(&claims, &state.config.jwt.secret) {
        Ok(token) => token,
        Err(e) => {
            tracing::error!("Access token generation failed: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to generate access token"
                })),
            )
                .into_response();
        }
    };

    // Generate new refresh token (rotation)
    let new_refresh_token = match generate_refresh_token(
        &mut state.redis.clone(),
        &user_data.user_id,
        &user_data.email,
        state.config.jwt.refresh_token_expiry_seconds,
    )
    .await
    {
        Ok(token) => token,
        Err(e) => {
            tracing::error!("Refresh token generation failed: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to generate refresh token"
                })),
            )
                .into_response();
        }
    };

    // Build response
    let response = RefreshResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: state.config.jwt.access_token_expiry_seconds,
        refresh_token: new_refresh_token,
    };

    (StatusCode::OK, Json(response)).into_response()
}
