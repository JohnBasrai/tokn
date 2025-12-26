// oauth2-server/src/handlers/authorize.rs

use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse},
};
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;

// ---

#[derive(Debug, Deserialize)]
pub struct AuthorizeQuery {
    // ---
    pub response_type: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub scope: Option<String>,
    pub state: Option<String>,
}

// ---

pub async fn authorize_handler(
    State(_pool): State<Arc<PgPool>>,
    Query(params): Query<AuthorizeQuery>,
) -> impl IntoResponse {
    // ---
    // TODO: Validate client_id exists in database
    // TODO: Validate redirect_uri matches client registration
    // TODO: Show consent page with approve/deny buttons

    // ---
    // For now, return simple consent page
    let html = format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>Authorization Request</title>
</head>
<body>
    <h1>Authorization Request</h1>
    <p>Application <strong>{}</strong> wants to access your account.</p>
    <p>Scopes: {}</p>
    <form method="POST" action="/oauth/authorize">
        <input type="hidden" name="client_id" value="{}">
        <input type="hidden" name="redirect_uri" value="{}">
        <input type="hidden" name="scope" value="{}">
        <input type="hidden" name="state" value="{}">
        <button type="submit" name="action" value="approve">Approve</button>
        <button type="submit" name="action" value="deny">Deny</button>
    </form>
</body>
</html>
"#,
        params.client_id,
        params.scope.as_deref().unwrap_or("profile"),
        params.client_id,
        params.redirect_uri,
        params.scope.as_deref().unwrap_or("profile"),
        params.state.as_deref().unwrap_or(""),
    );

    Html(html)
}
