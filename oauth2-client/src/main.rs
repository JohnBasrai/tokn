// oauth2-client/src/main.rs

use anyhow::Result;
use axum::{routing::get, Router};
use oauth2_client::Config;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// ---

#[tokio::main]
async fn main() -> Result<()> {
    // ---
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "oauth2_client=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_ansi(false))
        .init();

    // ---
    // Load configuration
    let config = Arc::new(Config::from_env()?);
    let bind_addr = config.bind_address();

    // ---
    tracing::info!("Starting oauth2-client on {}", bind_addr);

    // ---
    // Build router
    let app = Router::new()
        .route("/", get(oauth2_client::home_handler))
        .route("/login", get(oauth2_client::login_handler))
        .route("/callback", get(oauth2_client::callback_handler))
        .route("/profile", get(oauth2_client::profile_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(config);

    // ---
    // Start server
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("Listening on {}", bind_addr);

    axum::serve(listener, app).await?;

    Ok(())
}
