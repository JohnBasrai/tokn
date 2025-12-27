// jwt-service/src/redis_client.rs

//! Redis client for refresh token storage
//!
//! Manages Redis connections for storing and retrieving refresh tokens.

use anyhow::{Context, Result};
use redis::aio::ConnectionManager;
use redis::Client;

// ---

/// Create a Redis client connection.
///
/// Establishes a connection manager to Redis for storing refresh tokens.
///
/// # Arguments
///
/// - `redis_url` - Redis connection URL (e.g., "redis://127.0.0.1:6379")
///
/// # Returns
///
/// A Redis connection manager that can be cloned and shared across tasks.
///
/// # Errors
///
/// Returns error if:
/// - Redis URL is invalid
/// - Cannot connect to Redis server
/// - Redis server is not available
///
/// # Example
///
/// ```no_run
/// use jwt_service::create_redis_client;
///
/// # async fn example() -> anyhow::Result<()> {
/// let conn = create_redis_client("redis://127.0.0.1:6379").await?;
/// # Ok(())
/// # }
/// ```
pub async fn create_redis_client(redis_url: &str) -> Result<ConnectionManager> {
    // ---
    let client = Client::open(redis_url).context("Failed to create Redis client")?;

    ConnectionManager::new(client)
        .await
        .context("Failed to connect to Redis")
}
