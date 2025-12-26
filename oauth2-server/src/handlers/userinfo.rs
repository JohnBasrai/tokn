// oauth2-server/src/handlers/userinfo.rs

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};
use serde::Serialize;
use sqlx::PgPool;
use std::sync::Arc;

// ---

#[derive(Debug, Serialize)]
pub struct UserInfo {
    // ---
    sub: String, // User ID
    username: String,
}

// ---

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    // ---
    error: String,
}

// ---

pub async fn userinfo_handler(
    State(_pool): State<Arc<PgPool>>,
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
    let _token = auth_header.strip_prefix("Bearer ").unwrap_or("");

    // ---
    // TODO: Validate token exists in database
    // TODO: Check token hasn't expired
    // TODO: Fetch user_id from token
    // TODO: Fetch user info from users table

    // ---
    // Placeholder response
    Json(UserInfo {
        sub: "user_001".to_string(),
        username: "demo".to_string(),
    })
    .into_response()
}
