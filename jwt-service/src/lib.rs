// jwt-service/src/lib.rs

//! JWT token service library
//!
//! Provides JWT generation, validation, refresh, and revocation functionality.

mod claims;
mod config;
mod handlers;
mod redis_client;
mod refresh;
mod token;

use std::sync::Arc;

// ---

/// Type alias for Redis connection manager.
///
/// Provides async connection pooling for Redis operations.
pub type RedisConnection = redis::aio::ConnectionManager;

// ---

/// Application state shared across all handlers.
///
/// Contains configuration and Redis connection.
#[derive(Clone)]
pub struct AppState {
    // ---
    pub config: Arc<Config>,
    pub redis: RedisConnection,
}

// ---

pub use claims::Claims;
pub use config::Config;
pub use handlers::{generate_token_handler, refresh_token_handler, validate_token_handler};
pub use redis_client::create_redis_client;
pub use refresh::{generate_refresh_token, validate_refresh_token};
pub use token::{generate_token, validate_token};
