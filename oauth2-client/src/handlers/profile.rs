// oauth2-client/src/handlers/profile.rs

use axum::response::Html;

// ---

/// Displays the user profile page.
///
/// Currently shows a placeholder authenticated page.
///
/// # TODO
///
/// - Check for valid access token in Redis
/// - Fetch user info from oauth2-server userinfo endpoint
/// - Display actual user data instead of placeholder
pub async fn profile_handler() -> Html<&'static str> {
    // ---
    // TODO: Check for valid access token in Redis
    // TODO: Fetch user info from oauth2-server userinfo endpoint
    // TODO: Display actual user data

    // ---
    Html(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>Profile - OAuth2 Client Demo</title>
</head>
<body>
    <h1>Profile</h1>
    <p>You are authenticated!</p>
    <p><em>TODO: Display actual user information from userinfo endpoint</em></p>
    <a href="/">Back to Home</a>
</body>
</html>
"#,
    )
}
