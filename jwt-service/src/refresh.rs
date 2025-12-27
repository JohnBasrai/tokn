// jwt-service/src/refresh.rs

//! Refresh token generation and management
//!
//! Handles creation, storage, validation, and rotation of refresh tokens.

use anyhow::{Context, Result};
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, RedisError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---

/// Refresh token metadata stored in Redis.
///
/// Contains information needed to issue a new access token.
#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenData {
    // ---
    /// User ID this refresh token belongs to
    pub user_id: String,

    /// User email for claims generation
    pub email: String,
}

// ---

/// Generate and store a refresh token in Redis.
///
/// Creates a UUID refresh token and stores user information in Redis with TTL.
///
/// # Arguments
///
/// - `redis_conn` - Multiplexed Redis connection
/// - `user_id` - User identifier
/// - `email` - User email address
/// - `expiry_seconds` - Token expiry duration (e.g., 604800 = 7 days)
///
/// # Returns
///
/// The generated refresh token (UUID string).
///
/// # Storage Format
///
/// - Key: `refresh_token:{uuid}`
/// - Value: JSON `{ "user_id": "...", "email": "..." }`
/// - TTL: `expiry_seconds`
///
/// # Security
///
/// - Tokens are cryptographically random UUIDs (v4)
/// - Tokens automatically expire via Redis TTL
/// - Tokens are single-use (deleted on successful refresh)
///
/// # Errors
///
/// Returns error if Redis storage fails.
///
/// # Example
///
/// ```no_run
/// use jwt_service::{create_redis_client, generate_refresh_token};
///
/// # async fn example() -> anyhow::Result<()> {
/// let mut redis_conn = create_redis_client("redis://127.0.0.1:6379").await?;
/// let refresh_token = generate_refresh_token(
///     &mut redis_conn,
///     "user_123",
///     "user@example.com",
///     604800  // 7 days
/// ).await?;
/// # Ok(())
/// # }
/// ```
pub async fn generate_refresh_token(
    redis_conn: &mut ConnectionManager,
    user_id: &str,
    email: &str,
    expiry_seconds: i64,
) -> Result<String> {
    // ---
    // Generate cryptographically random UUID
    let refresh_token = Uuid::new_v4().to_string();
    let redis_key = format!("refresh_token:{}", refresh_token);

    // Store user data
    let token_data = RefreshTokenData {
        user_id: user_id.to_string(),
        email: email.to_string(),
    };

    let token_json =
        serde_json::to_string(&token_data).context("Failed to serialize refresh token data")?;

    // Store with TTL
    redis_conn
        .set_ex::<_, _, ()>(&redis_key, token_json, expiry_seconds as u64)
        .await
        .context("Failed to store refresh token in Redis")?;

    Ok(refresh_token)
}

// ---

/// Validate and consume a refresh token.
///
/// Retrieves user data from Redis and **deletes** the refresh token (one-time use).
///
/// # Arguments
///
/// - `redis_conn` - Multiplexed Redis connection
/// - `refresh_token` - The refresh token UUID to validate
///
/// # Returns
///
/// The user data if the token is valid.
///
/// # Security - Token Rotation
///
/// This function implements **refresh token rotation**:
/// 1. Validates the token exists in Redis
/// 2. Retrieves the user data
/// 3. **Deletes the token** (prevents reuse)
///
/// This prevents replay attacks - if an attacker steals a refresh token,
/// it can only be used once. The next legitimate refresh will fail,
/// alerting the user to revoke all sessions.
///
/// # Errors
///
/// Returns error if:
/// - Token doesn't exist in Redis (expired or already used)
/// - Token data is invalid JSON
/// - Redis operation fails
///
/// # Example
///
/// ```no_run
/// use jwt_service::{create_redis_client, validate_refresh_token};
///
/// # async fn example() -> anyhow::Result<()> {
/// let mut redis_conn = create_redis_client("redis://127.0.0.1:6379").await?;
/// let user_data = validate_refresh_token(
///     &mut redis_conn,
///     "f47ac10b-58cc-4372-a567-0e02b2c3d479"
/// ).await?;
/// println!("Valid refresh token for user: {}", user_data.user_id);
/// # Ok(())
/// # }
/// ```
pub async fn validate_refresh_token(
    redis_conn: &mut ConnectionManager,
    refresh_token: &str,
) -> Result<RefreshTokenData> {
    // ---
    let redis_key = format!("refresh_token:{}", refresh_token);

    // Get token data
    let token_json: String = redis_conn
        .get(&redis_key)
        .await
        .map_err(|e: RedisError| anyhow::anyhow!("Invalid or expired refresh token: {}", e))?;

    // Delete token (one-time use - rotation)
    redis_conn
        .del::<_, ()>(&redis_key)
        .await
        .context("Failed to delete refresh token")?;

    // Parse user data
    let token_data: RefreshTokenData =
        serde_json::from_str(&token_json).context("Invalid refresh token data format")?;

    Ok(token_data)
}
