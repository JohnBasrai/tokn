// oauth2-client/src/handlers/callback.rs

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthorizationCode, ClientId, ClientSecret,
    RedirectUrl, TokenResponse, TokenUrl,
};
use serde::Deserialize;
use std::sync::Arc;

// ---

use crate::Config;

// ---

#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    // ---
    code: String,
    _state: Option<String>,
}

// ---

pub async fn callback_handler(
    State(config): State<Arc<Config>>,
    Query(params): Query<CallbackQuery>,
) -> impl IntoResponse {
    // ---
    // TODO: Validate CSRF state token from Redis

    // ---
    // Build OAuth2 client
    let client = BasicClient::new(
        ClientId::new(config.oauth2.client_id.clone()),
        Some(ClientSecret::new(config.oauth2.client_secret.clone())),
        oauth2::AuthUrl::new(config.oauth2.authorize_url.clone()).expect("Invalid authorize URL"),
        Some(TokenUrl::new(config.oauth2.token_url.clone()).expect("Invalid token URL")),
    )
    .set_redirect_uri(
        RedirectUrl::new(config.oauth2.redirect_uri.clone()).expect("Invalid redirect URI"),
    );

    // ---
    // Exchange authorization code for access token
    let token_result = client
        .exchange_code(AuthorizationCode::new(params.code))
        .request_async(async_http_client)
        .await;

    // ---
    match token_result {
        // ---
        Ok(token) => {
            // ---
            let _access_token = token.access_token().secret();

            // ---
            // TODO: Store access_token in Redis with expiry
            // TODO: Fetch user info from userinfo endpoint

            // ---
            // For now, redirect to profile page
            Redirect::to("/profile")
        }
        Err(err) => {
            // ---
            tracing::error!("Token exchange failed: {:?}", err);
            Redirect::to("/?error=token_exchange_failed")
        }
    }
}
