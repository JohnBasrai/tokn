// oauth2-server/src/handlers/token.rs

use axum::{
    extract::State,
    response::{IntoResponse, Json},
    Form,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;

// ---

#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    // ---
    _grant_type: String,
    _code: String,
    _redirect_uri: String,
    _client_id: String,
    _client_secret: String,
}

// ---

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    // ---
    access_token: String,
    token_type: String,
    expires_in: i64,
}

// ---

#[derive(Debug, Serialize)]
pub struct TokenError {
    // ---
    error: String,
    error_description: String,
}

// ---

pub async fn token_handler(
    State(_pool): State<Arc<PgPool>>,
    Form(_params): Form<TokenRequest>,
) -> impl IntoResponse {
    // ---
    // TODO: Validate grant_type is "authorization_code"
    // TODO: Validate client credentials
    // TODO: Validate authorization code exists and hasn't expired
    // TODO: Generate access token
    // TODO: Store access token in database
    // TODO: Delete used authorization code

    // ---
    // Placeholder response
    Json(TokenResponse {
        access_token: "placeholder_token_12345".to_string(),
        token_type: "Bearer".to_string(),
        expires_in: 3600,
    })
}
