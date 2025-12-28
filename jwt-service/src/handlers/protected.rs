// jwt-service/src/handlers/protected.rs

//! Protected route demonstrating JWT authentication middleware
//!
//! This module showcases how to protect API endpoints using JWT tokens.
//! Routes require valid, unexpired, non-revoked tokens with proper signatures.

use crate::{is_token_revoked, token::validate_token, AppState, Claims};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};

// ---

/// User profile information returned from protected endpoints
#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfile {
    // ---
    pub message: String,
    pub user_id: String,
    pub email: String,
    pub token_issued_at: i64,
    pub token_expires_at: i64,
}

// ---

/// Extract and validate JWT from Authorization header
///
/// This middleware intercepts all requests to protected routes and:
/// 1. Extracts the Bearer token from Authorization header
/// 2. Validates the token signature
/// 3. Checks token expiration
/// 4. Checks token revocation status
/// 5. Injects validated claims into request extensions
///
/// # Security
/// - Tokens must use valid HS256 signature
/// - Tokens must not be expired (checked against server time)
/// - Revoked tokens are rejected via Redis blacklist check
/// - Malformed Authorization headers are rejected
///
/// # Header Format
///
/// ```text
/// Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
/// ```
///
/// # Errors
///
/// Returns `401 Unauthorized` if:
/// - Authorization header is missing
/// - Header doesn't start with "Bearer "
/// - Token signature is invalid
/// - Token has expired
/// - Token has been revoked
///
/// Returns `500 Internal Server Error` if:
/// - Redis connection fails during revocation check
async fn jwt_auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // ---
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // ---
    // Verify Bearer token format
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // ---
    // Validate token and extract claims
    let claims = validate_token(token, &state.config.jwt.secret).map_err(|e| {
        tracing::warn!("Token validation failed: {:?}", e);
        StatusCode::UNAUTHORIZED
    })?;

    // ---
    // Check if token is revoked
    let is_revoked = is_token_revoked(&mut state.redis.clone(), &claims.jti)
        .await
        .map_err(|e| {
            tracing::error!("Failed to check token revocation status: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if is_revoked {
        tracing::warn!("Revoked token attempted access (jti={})", claims.jti);
        return Err(StatusCode::UNAUTHORIZED);
    }

    // ---
    // Token is valid - inject claims into request extensions
    request.extensions_mut().insert(claims);

    // ---
    // Continue to the actual handler
    Ok(next.run(request).await)
}

// ---

/// Protected endpoint handler - requires valid JWT
///
/// Demonstrates how to access extracted claims from middleware.
/// The JWT middleware runs first, validates the token, and injects
/// the `Claims` into request extensions.
///
/// # Security
///
/// This endpoint is only reachable if:
/// - Valid Authorization header is present
/// - Token signature is valid (HS256)
/// - Token has not expired
/// - Token has not been revoked
///
/// # Response
///
/// ```json
/// {
///   "message": "Access granted",
///   "user_id": "user_12345",
///   "email": "john@example.com",
///   "token_issued_at": 1703000334,
///   "token_expires_at": 1703001234
/// }
/// ```
async fn protected_handler(
    axum::extract::Extension(claims): axum::extract::Extension<Claims>,
) -> impl IntoResponse {
    // ---
    // Claims are already validated and extracted by middleware
    let profile = UserProfile {
        message: "Access granted".to_string(),
        user_id: claims.sub.clone(),
        email: claims.email.clone(),
        token_issued_at: claims.iat as i64,
        token_expires_at: claims.exp as i64,
    };

    (StatusCode::OK, Json(profile))
}

// ---

/// Create router with protected routes
///
/// This demonstrates how to add authentication middleware to specific routes.
/// The middleware only runs for paths under this router.
///
/// # Example
///
/// ```bash
/// # Get a token first
/// TOKEN=$(curl -X POST http://localhost:8083/auth/token \
///   -H "Content-Type: application/json" \
///   -d '{"user_id": "test_user", "email": "test@example.com"}' \
///   | jq -r '.access_token')
///
/// # Access protected endpoint
/// curl http://localhost:8083/protected \
///   -H "Authorization: Bearer $TOKEN"
/// ```
pub fn protected_routes(state: AppState) -> Router<AppState> {
    // ---
    Router::new()
        .route("/protected", get(protected_handler))
        .layer(middleware::from_fn_with_state(state, jwt_auth_middleware))
}
