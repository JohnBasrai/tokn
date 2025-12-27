// jwt-service/src/main.rs

//! JWT Service - Token generation, validation, and management
//!
//! Provides a standalone JWT authentication service with:
//! - Token generation with custom claims
//! - Signature validation and expiry checking
//! - Refresh token rotation
//! - Token revocation (blacklisting)

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use jwt_service::{generate_token_handler, validate_token_handler, Config};
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // ---
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
    let config = Arc::new(Config::from_env()?);

    info!(
        "Starting JWT service on {}:{}",
        config.server.host, config.server.port
    );

    // Build application router
    let app = Router::new()
        .route("/", get(|| async { "JWT Service - Ready" }))
        .route("/health", get(|| async { "OK" }))
        .route("/auth/token", post(generate_token_handler))
        .route("/auth/validate", post(validate_token_handler))
        .with_state(config.clone());

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("JWT service listening on {}", addr);
    info!("Endpoints:");
    info!("  POST /auth/token - Generate JWT token");
    info!("  POST /auth/validate - Validate JWT token");

    axum::serve(listener, app).await?;

    Ok(())
}
