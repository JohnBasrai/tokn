// oauth2-server/src/handlers/authorize_post.rs

use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
    Form,
};
use chrono::{Duration, Utc};
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

// ---

/// Form data submitted from the authorization consent page.
///
/// Contains the client's authorization request parameters and the user's decision (approve/deny).
#[derive(Debug, Deserialize)]
pub struct AuthorizeForm {
    // ---
    pub client_id: String,
    pub redirect_uri: String,
    pub scope: String,
    pub state: String,
    pub action: String, // "approve" or "deny"
}

// ---

/// Processes the user's authorization decision and generates an authorization code.
///
/// This implements the authorization response step of the OAuth2 authorization code flow (RFC 6749 §4.1.2).
/// If the user approves, generates an authorization code and redirects back to the client.
///
/// # Security
///
/// - Generates cryptographically random authorization code (UUID v4)
/// - Sets 5-minute expiration on authorization codes
/// - Stores code with associated client_id and redirect_uri for validation during token exchange
/// - TODO: Validate client_id exists in database
/// - TODO: Validate redirect_uri matches client registration
/// - TODO: Get actual user_id from authenticated session instead of hardcoded value
///
/// # OAuth2 Flow
///
/// If approved:
/// 1. Generate authorization code
/// 2. Store code in database with expiration
/// 3. Redirect to client's redirect_uri with code and state
///
/// If denied:
/// 1. Redirect to client's redirect_uri with error=access_denied
///
/// # Errors
///
/// Returns redirect with error=server_error if database operations fail.
pub async fn authorize_post_handler(
    State(pool): State<Arc<PgPool>>,
    Form(form): Form<AuthorizeForm>,
) -> impl IntoResponse {
    // ---
    // If user denied, redirect with error
    if form.action == "deny" {
        let error_url = format!(
            "{}?error=access_denied&state={}",
            form.redirect_uri, form.state
        );
        return Redirect::to(&error_url);
    }

    // ---
    // Generate authorization code
    let code = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::minutes(5);

    // ---
    // Store authorization code in database
    // TODO: Get actual user_id from session (hardcoded for now)
    let user_id = "user_001";

    let result = sqlx::query!(
        r#"
        INSERT INTO authorization_codes (code, client_id, user_id, redirect_uri, scope, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        code,
        form.client_id,
        user_id,
        form.redirect_uri,
        form.scope,
        expires_at.naive_utc()
    )
    .execute(pool.as_ref())
    .await;

    // ---
    match result {
        Ok(_) => {
            // Redirect back to client with authorization code
            let callback_url = format!("{}?code={}&state={}", form.redirect_uri, code, form.state);
            Redirect::to(&callback_url)
        }
        Err(e) => {
            tracing::error!("Failed to store authorization code: {:?}", e);
            let error_url = format!(
                "{}?error=server_error&state={}",
                form.redirect_uri, form.state
            );
            Redirect::to(&error_url)
        }
    }
}
