// oauth2-server/src/handlers/token.rs

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

// ---

/// OAuth2 token request parameters.
///
/// Sent by the client to exchange an authorization code for an access token (RFC 6749 §4.1.3).
#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    // ---
    pub grant_type: String,
    pub code: String,
    pub redirect_uri: String,
    pub client_id: String,
    pub client_secret: String,
}

// ---

/// OAuth2 token response.
///
/// Returned to the client after successful token exchange (RFC 6749 §4.1.4).
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    // ---
    access_token: String,
    token_type: String,
    expires_in: i64,
}

// ---

/// OAuth2 token error response.
///
/// Returned when token exchange fails (RFC 6749 §5.2).
#[derive(Debug, Serialize)]
pub struct TokenError {
    // ---
    error: String,
    error_description: String,
}

// ---

/// Exchanges an authorization code for an access token.
///
/// This implements the token endpoint of the OAuth2 authorization code flow (RFC 6749 §4.1.3).
/// The client exchanges a one-time authorization code for an access token that can be used
/// to access protected resources.
///
/// # Security
///
/// - Validates client credentials (client_id and client_secret)
/// - Verifies authorization code exists and hasn't been used
/// - Checks authorization code hasn't expired (5-minute TTL)
/// - Validates redirect_uri matches the one used during authorization
/// - Invalidates authorization code after successful exchange (one-time use)
/// - Generates cryptographically random access token (UUID v4)
/// - Sets 1-hour expiration on access tokens
///
/// # OAuth2 Flow
///
/// 1. Parse and validate token request parameters
/// 2. Validate client credentials against database
/// 3. Fetch and validate authorization code
/// 4. Check code expiration and redirect_uri match
/// 5. Generate access token with expiration
/// 6. Store access token in database
/// 7. Delete used authorization code
/// 8. Return access token to client
///
/// # Errors
///
/// Returns JSON error response with appropriate HTTP status code:
/// - 400 BAD_REQUEST: Malformed request, unsupported grant type, invalid/expired code, redirect_uri mismatch
/// - 401 UNAUTHORIZED: Invalid client credentials, client not found
/// - 500 INTERNAL_SERVER_ERROR: Database errors, token generation failures
///
/// Error responses follow RFC 6749 §5.2 format with `error` and `error_description` fields.
pub async fn token_handler(
    State(pool): State<Arc<PgPool>>,
    body: String, // Capture raw body first
) -> impl IntoResponse {
    // ---
    tracing::debug!("Raw token request body: {body}");

    // Then manually deserialize with better error context
    let params: TokenRequest = match serde_urlencoded::from_str(&body) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Failed to parse token request: {:?}\nBody was: {body}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(TokenError {
                    error: "invalid_request".to_string(),
                    error_description: format!("Malformed request: {e}"),
                }),
            )
                .into_response();
        }
    };

    tracing::debug!("Received token request: {:?}", params);

    // Validate grant type
    if params.grant_type != "authorization_code" {
        return (
            StatusCode::BAD_REQUEST,
            Json(TokenError {
                error: "unsupported_grant_type".to_string(),
                error_description: "Only authorization_code grant type is supported".to_string(),
            }),
        )
            .into_response();
    }

    // ---
    // Validate client credentials
    let client_result = sqlx::query!(
        r#"
        SELECT client_secret, redirect_uri
        FROM clients
        WHERE client_id = $1
        "#,
        params.client_id
    )
    .fetch_optional(pool.as_ref())
    .await;

    let client = match client_result {
        Ok(Some(c)) => c,
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(TokenError {
                    error: "invalid_client".to_string(),
                    error_description: "Client not found".to_string(),
                }),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!("Database error checking client: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(TokenError {
                    error: "server_error".to_string(),
                    error_description: "Internal server error".to_string(),
                }),
            )
                .into_response();
        }
    };

    // ---
    // Verify client secret
    if client.client_secret != params.client_secret {
        return (
            StatusCode::UNAUTHORIZED,
            Json(TokenError {
                error: "invalid_client".to_string(),
                error_description: "Invalid client credentials".to_string(),
            }),
        )
            .into_response();
    }

    // ---
    // Fetch authorization code
    let code_result = sqlx::query!(
        r#"
        SELECT user_id, redirect_uri, scope, expires_at
        FROM authorization_codes
        WHERE code = $1 AND client_id = $2
        "#,
        params.code,
        params.client_id
    )
    .fetch_optional(pool.as_ref())
    .await;

    let auth_code = match code_result {
        Ok(Some(c)) => c,
        Ok(None) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(TokenError {
                    error: "invalid_grant".to_string(),
                    error_description: "Authorization code not found".to_string(),
                }),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!("Database error fetching auth code: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(TokenError {
                    error: "server_error".to_string(),
                    error_description: "Internal server error".to_string(),
                }),
            )
                .into_response();
        }
    };

    // ---
    // Check code hasn't expired
    if auth_code.expires_at < Utc::now().naive_utc() {
        return (
            StatusCode::BAD_REQUEST,
            Json(TokenError {
                error: "invalid_grant".to_string(),
                error_description: "Authorization code has expired".to_string(),
            }),
        )
            .into_response();
    }

    // ---
    // Verify redirect_uri matches
    if auth_code.redirect_uri != params.redirect_uri {
        return (
            StatusCode::BAD_REQUEST,
            Json(TokenError {
                error: "invalid_grant".to_string(),
                error_description: "Redirect URI mismatch".to_string(),
            }),
        )
            .into_response();
    }

    // ---
    // Generate access token
    let access_token = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::hours(1);

    // ---
    // Store access token
    let insert_result = sqlx::query!(
        r#"
        INSERT INTO access_tokens (token, client_id, user_id, scope, expires_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        access_token,
        params.client_id,
        auth_code.user_id,
        auth_code.scope,
        expires_at.naive_utc()
    )
    .execute(pool.as_ref())
    .await;

    if let Err(e) = insert_result {
        tracing::error!("Failed to store access token: {:?}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(TokenError {
                error: "server_error".to_string(),
                error_description: "Failed to generate token".to_string(),
            }),
        )
            .into_response();
    }

    // ---
    // Delete used authorization code
    let _ = sqlx::query!(
        r#"
        DELETE FROM authorization_codes WHERE code = $1
        "#,
        params.code
    )
    .execute(pool.as_ref())
    .await;

    // ---
    // Return success
    Json(TokenResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600, // 1 hour
    })
    .into_response()
}
