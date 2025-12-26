// oauth2-client/src/handlers/callback.rs

use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Redirect},
};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, RedirectUrl, TokenResponse, TokenUrl,
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
        AuthUrl::new(config.oauth2.authorize_url.clone()).expect("Invalid authorize URL"),
        Some(TokenUrl::new(config.oauth2.token_url.clone()).expect("Invalid token URL")),
    )
    .set_redirect_uri(
        RedirectUrl::new(config.oauth2.redirect_uri.clone()).expect("Invalid redirect URI"),
    )
    .set_auth_type(oauth2::AuthType::RequestBody); // Add this line

    // ---
    // Exchange authorization code for access token
    let token_result = client
        .exchange_code(AuthorizationCode::new(params.code))
        .request_async(async_http_client)
        .await;

    // ---
    let access_token = match token_result {
        // ---
        Ok(token) => token.access_token().secret().to_string(),
        Err(err) => {
            // ---
            tracing::error!("Token exchange failed: {:?}", err);
            return Redirect::to("/?error=token_exchange_failed").into_response();
        }
    };

    // ---
    // Fetch user info from userinfo endpoint
    let http_client = reqwest::Client::new();
    let userinfo_result = http_client
        .get(&config.oauth2.userinfo_url)
        .bearer_auth(&access_token)
        .send()
        .await;

    let userinfo_response = match userinfo_result {
        Ok(resp) => resp,
        Err(err) => {
            tracing::error!("Userinfo request failed: {:?}", err);
            return Redirect::to("/?error=userinfo_failed").into_response();
        }
    };

    // ---
    let userinfo_json = match userinfo_response.json::<serde_json::Value>().await {
        Ok(json) => json,
        Err(err) => {
            tracing::error!("Failed to parse userinfo: {:?}", err);
            return Redirect::to("/?error=userinfo_parse_failed").into_response();
        }
    };

    // ---
    // Extract username from userinfo
    let username = userinfo_json
        .get("username")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown");

    // ---
    // Display success page with user info
    let html = format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>Logged In - OAuth2 Client Demo</title>
</head>
<body>
    <h1>Successfully Authenticated!</h1>
    <p>Welcome, <strong>{username}</strong></p>
    <p>Access Token: <code>{access_token}</code></p>
    <a href="/">Back to Home</a>
</body>
</html>
"#,
    );

    Html(html).into_response()
}
