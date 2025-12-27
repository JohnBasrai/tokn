// jwt-service/src/handlers/revoke.rs

//! Token revocation endpoint
//!
//! Handles POST /auth/revoke - revokes (blacklists) JWT tokens

use crate::{revoke_token, token::validate_token, AppState};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};

// ---

/// Request payload for token revocation.
///
/// Client sends the JWT token to revoke.
#[derive(Debug, Deserialize)]
pub struct RevokeRequest {
    // ---
    /// The JWT token to revoke
    pub token: String,
}

// ---

/// Response confirming token revocation.
#[derive(Debug, Serialize)]
pub struct RevokeResponse {
    // ---
    /// Confirmation message
    message: String,

    /// The revoked token's JTI
    jti: String,
}

// ---

/// Revoke (blacklist) a JWT token.
///
/// This endpoint validates the token and adds its JTI to the Redis blacklist.
/// The token will be rejected by future validation attempts.
///
/// # Request
///
/// ```json
/// {
///   "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
/// }
/// ```
///
/// # Response (200 OK)
///
/// ```json
/// {
///   "message": "Token revoked successfully",
///   "jti": "f47ac10b-58cc-4372-a567-0e02b2c3d479"
/// }
/// ```
///
/// # Response (401 Unauthorized) - Invalid Token
///
/// ```json
/// {
///   "error": "Invalid token"
/// }
/// ```
///
/// # Security
///
/// - Token must be valid (signature + expiry) to be revoked
/// - JTI is stored in Redis with TTL matching token expiry
/// - Already-expired tokens cannot be revoked (no point)
/// - Revoked tokens are rejected by POST /auth/validate
///
/// # Use Cases
///
/// - **User logout**: Client revokes token on logout
/// - **Security incident**: Revoke compromised token immediately
/// - **Password change**: Revoke all existing tokens
/// - **Admin action**: Force logout specific users
///
/// # Revocation Strategy
///
/// This implements **JWT blacklisting**:
/// - Tokens are stateless (can't be "deleted")
/// - Instead, we track revoked tokens in Redis
/// - Validation checks blacklist before accepting token
/// - Blacklist entries expire when token would expire anyway
///
/// # Alternative: Short-Lived Tokens
///
/// For high-security applications, prefer:
/// - Very short access token expiry (5 minutes)
/// - Refresh token rotation (see `src/handlers/refresh.rs` and `src/refresh.rs`)
/// - No revocation needed (tokens expire quickly)
///
/// # Errors
///
/// Returns 401 Unauthorized if:
/// - Token signature is invalid
/// - Token has expired
/// - Token format is malformed
///
/// Returns 500 Internal Server Error if Redis storage fails.
pub async fn revoke_token_handler(
    State(state): State<AppState>,
    Json(req): Json<RevokeRequest>,
) -> impl IntoResponse {
    // ---
    // Validate token first (must be valid to revoke)
    let claims = match validate_token(&req.token, &state.config.jwt.secret) {
        Ok(claims) => claims,
        Err(e) => {
            tracing::debug!("Cannot revoke invalid token: {}", e);
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Invalid token"
                })),
            )
                .into_response();
        }
    };

    // Calculate remaining TTL (time until expiration)
    let now = Utc::now().timestamp() as usize;
    let remaining_ttl = if claims.exp > now {
        (claims.exp - now) as i64
    } else {
        // Token already expired, no need to revoke
        tracing::debug!("Token already expired, not revoking");
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Token already expired"
            })),
        )
            .into_response();
    };

    // Revoke the token (add JTI to blacklist)
    if let Err(e) = revoke_token(&mut state.redis.clone(), &claims.jti, remaining_ttl).await {
        tracing::error!("Failed to revoke token: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "Failed to revoke token"
            })),
        )
            .into_response();
    }

    tracing::info!("Token revoked: jti={}", claims.jti);

    let response = RevokeResponse {
        message: "Token revoked successfully".to_string(),
        jti: claims.jti,
    };

    (StatusCode::OK, Json(response)).into_response()
}
