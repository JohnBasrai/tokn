// oauth2-client/src/handlers/login.rs

use axum::{
    extract::State,
    response::{IntoResponse, Redirect},
};

use oauth2::{AuthUrl, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope, TokenUrl};
use std::sync::Arc;

// ---

use crate::Config;

// ---

/// Initiates the OAuth2 authorization code flow.
///
/// This implements the authorization request step of the OAuth2 authorization code flow (RFC 6749 §4.1.1).
/// Constructs an authorization URL and redirects the user to the authorization server.
///
/// # Security
///
/// - Generates a CSRF token (currently not validated - TODO)
/// - Requests the "profile" scope
/// - Uses HTTPS redirect to authorization server
///
/// # OAuth2 Flow
///
/// 1. Constructs OAuth2 client with credentials
/// 2. Generates authorization URL with:
///    - client_id
///    - redirect_uri (callback endpoint)
///    - scope (requested permissions)
///    - state (CSRF token)
/// 3. Redirects user to authorization server for consent
///
/// After user approval, the authorization server redirects back to the callback handler.
pub async fn login_handler(State(config): State<Arc<Config>>) -> impl IntoResponse {
    // ---
    // Build OAuth2 client
    let client = oauth2::basic::BasicClient::new(
        ClientId::new(config.oauth2.client_id.clone()),
        Some(ClientSecret::new(config.oauth2.client_secret.clone())),
        AuthUrl::new(config.oauth2.authorize_url.clone()).expect("Invalid authorize URL"),
        Some(TokenUrl::new(config.oauth2.token_url.clone()).expect("Invalid token URL")),
    )
    .set_redirect_uri(
        RedirectUrl::new(config.oauth2.redirect_uri.clone()).expect("Invalid redirect URI"),
    );

    // ---
    // Generate authorization URL
    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("profile".to_string()))
        .url();

    // ---
    // TODO: Store CSRF token in Redis for validation in callback

    // ---
    // Redirect to authorization server
    Redirect::to(auth_url.as_str())
}
