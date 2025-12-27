// oauth2-server/src/main.rs

use anyhow::Result;
use axum::{
    http::StatusCode,
    routing::{get, post},
    Router,
};
use oauth2_server::Config;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// ---

/// Root endpoint - service info
///
/// **Security Note:** This endpoint reveals service information and available endpoints.
/// In production, consider removing this or placing it behind authentication to avoid
/// information disclosure to potential attackers.
async fn root_handler() -> (StatusCode, &'static str) {
    // ---
    (
        StatusCode::OK,
        "oauth2-server v0.1.0\n\
         \n\
         Available endpoints:\n\
         - GET/POST /oauth/authorize - Authorization endpoint\n\
         - POST /oauth/token - Token exchange endpoint\n\
         - GET /oauth/userinfo - User information endpoint\n",
    )
}

// ---

#[tokio::main]
async fn main() -> Result<()> {
    // ---
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "oauth2_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_ansi(false))
        .init();

    // ---
    // Load configuration
    let config = Arc::new(Config::from_env()?);
    let bind_addr = config.bind_address();

    // ---
    // Create database pool
    let pool = Arc::new(oauth2_server::create_pool(&config.database.url).await?);

    // ---
    tracing::info!("Starting oauth2-server on {}", bind_addr);

    // ---
    // Build router
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/oauth/authorize", get(oauth2_server::authorize_handler))
        .route(
            "/oauth/authorize",
            post(oauth2_server::authorize_post_handler),
        )
        .route("/oauth/token", post(oauth2_server::token_handler))
        .route("/oauth/userinfo", get(oauth2_server::userinfo_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(pool);

    // ---
    // Start server
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("Listening on {}", bind_addr);

    axum::serve(listener, app).await?;

    Ok(())
}
