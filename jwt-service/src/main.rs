// jwt-service/src/main.rs

//! JWT Service - Token generation, validation, and management
//!
//! Provides a standalone JWT authentication service with:
//! - Token generation with custom claims
//! - Signature validation and expiry checking
//! - Refresh token rotation
//! - Token revocation (blacklisting)

use anyhow::Result;
use axum::{routing::get, Router};
use jwt_service::Config;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "jwt_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    dotenvy::dotenv().ok();
    let config = Config::from_env()?;

    info!(
        "Starting JWT service on {}:{}",
        config.server.host, config.server.port
    );

    // Build application router
    let app = Router::new()
        .route("/", get(|| async { "JWT Service - Ready" }))
        .route("/health", get(|| async { "OK" }));

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("JWT service listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
