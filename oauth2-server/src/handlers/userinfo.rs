// oauth2-server/src/handlers/userinfo.rs

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};
use chrono::Utc;
use serde::Serialize;
use sqlx::PgPool;
use std::sync::Arc;

// ---

/// User information response.
///
/// Returns the authenticated user's profile information (OIDC UserInfo endpoint).
#[derive(Debug, Serialize)]
pub struct UserInfo {
    // ---
    sub: String, // User ID (subject)
    username: String,
}

// ---

/// Error response for userinfo endpoint.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    // ---
    error: String,
}

// ---

/// Returns user profile information for a valid access token.
///
/// This implements the OIDC UserInfo endpoint that returns claims about the authenticated user.
/// Clients use this endpoint to fetch user profile data after obtaining an access token.
///
/// # Security
///
/// - Requires valid Bearer token in Authorization header
/// - Validates token exists in database
/// - Checks token hasn't expired (1-hour TTL)
/// - Returns 401 UNAUTHORIZED for missing/invalid/expired tokens
/// - Only returns user data associated with the token's user_id
///
/// # OAuth2 Flow
///
/// 1. Extract Bearer token from Authorization header
/// 2. Validate token exists and hasn't expired
/// 3. Fetch user information using the token's user_id
/// 4. Return user profile as JSON
///
/// # Errors
///
/// Returns JSON error response with appropriate HTTP status code:
/// - 401 UNAUTHORIZED: Missing/invalid Authorization header, invalid/expired token
/// - 404 NOT_FOUND: User not found in database
/// - 500 INTERNAL_SERVER_ERROR: Database errors
pub async fn userinfo_handler(
    State(pool): State<Arc<PgPool>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // ---
    // Extract Bearer token from Authorization header
    let auth_header = match headers.get("Authorization") {
        Some(h) => h.to_str().unwrap_or(""),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Missing Authorization header".to_string(),
                }),
            )
                .into_response();
        }
    };

    // ---
    let token = match auth_header.strip_prefix("Bearer ") {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Invalid Authorization header format".to_string(),
                }),
            )
                .into_response();
        }
    };

    // ---
    // Validate token and fetch user_id
    let token_result = sqlx::query!(
        r#"
        SELECT user_id, expires_at
        FROM access_tokens
        WHERE token = $1
        "#,
        token
    )
    .fetch_optional(pool.as_ref())
    .await;

    let access_token = match token_result {
        Ok(Some(t)) => t,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Invalid token".to_string(),
                }),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!("Database error fetching token: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Internal server error".to_string(),
                }),
            )
                .into_response();
        }
    };

    // ---
    // Check token hasn't expired
    if access_token.expires_at < Utc::now().naive_utc() {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Token has expired".to_string(),
            }),
        )
            .into_response();
    }

    // ---
    // Fetch user info
    let user_result = sqlx::query!(
        r#"
        SELECT user_id, username
        FROM users
        WHERE user_id = $1
        "#,
        access_token.user_id
    )
    .fetch_optional(pool.as_ref())
    .await;

    let user = match user_result {
        Ok(Some(u)) => u,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "User not found".to_string(),
                }),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!("Database error fetching user: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Internal server error".to_string(),
                }),
            )
                .into_response();
        }
    };

    // ---
    // Return user info
    Json(UserInfo {
        sub: user.user_id,
        username: user.username,
    })
    .into_response()
}
