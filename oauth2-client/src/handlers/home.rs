// oauth2-client/src/handlers/home.rs

use axum::response::Html;

// ---

/// Displays the OAuth2 client demo home page.
///
/// Shows a simple landing page with a login button that initiates the OAuth2 authorization code flow.
pub async fn home_handler() -> Html<&'static str> {
    // ---
    Html(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>OAuth2 Client Demo</title>
</head>
<body>
    <h1>OAuth2 Client Demo</h1>
    <p>This demo shows OAuth2 authorization code flow.</p>
    <a href="/login">
        <button>Login with OAuth2</button>
    </a>
</body>
</html>
"#,
    )
}
