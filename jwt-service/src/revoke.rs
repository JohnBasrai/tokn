// jwt-service/src/revoke.rs

//! Token revocation (blacklisting)
//!
//! Manages token revocation by storing JWT IDs in Redis.

use anyhow::{Context, Result};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;

// ---

/// Revoke a JWT token by adding its JTI to the blacklist.
///
/// Stores the token's JTI (JWT ID) in Redis with a TTL matching the token's
/// remaining lifetime. This prevents already-expired tokens from accumulating
/// in the blacklist.
///
/// # Arguments
///
/// - `redis_conn` - Redis connection manager
/// - `jti` - JWT ID (unique identifier from token claims)
/// - `expiry_seconds` - Token's remaining TTL (time until expiration)
///
/// # Storage Format
///
/// - Key: `blacklist:jti:{jti}`
/// - Value: `"revoked"` (placeholder, existence check is what matters)
/// - TTL: `expiry_seconds`
///
/// # Why TTL Matches Token Expiry
///
/// Once a token expires naturally, there's no need to keep it blacklisted.
/// The TTL ensures the blacklist only contains tokens that could still be
/// validated if not revoked.
///
/// # Use Cases
///
/// - User logout (revoke all their tokens)
/// - Security incident (revoke compromised token)
/// - Password change (revoke all existing sessions)
/// - Admin action (force logout specific user)
///
/// # Example
///
/// ```no_run
/// use jwt_service::{create_redis_client, revoke_token};
///
/// # async fn example() -> anyhow::Result<()> {
/// let mut redis_conn = create_redis_client("redis://127.0.0.1:6379").await?;
/// revoke_token(&mut redis_conn, "f47ac10b-...", 900).await?;
/// # Ok(())
/// # }
/// ```
pub async fn revoke_token(
    redis_conn: &mut ConnectionManager,
    jti: &str,
    expiry_seconds: i64,
) -> Result<()> {
    // ---
    let redis_key = format!("blacklist:jti:{}", jti);

    // Store with TTL matching token expiry
    redis_conn
        .set_ex::<_, _, ()>(&redis_key, "revoked", expiry_seconds as u64)
        .await
        .context("Failed to revoke token in Redis")?;

    Ok(())
}

// ---

/// Check if a JWT token is revoked.
///
/// Queries Redis to see if the token's JTI exists in the blacklist.
///
/// # Arguments
///
/// - `redis_conn` - Redis connection manager
/// - `jti` - JWT ID to check
///
/// # Returns
///
/// - `true` if the token is revoked (exists in blacklist)
/// - `false` if the token is not revoked
///
/// # Security Note
///
/// This check should be performed AFTER signature and expiry validation.
/// Don't waste Redis queries on invalid tokens.
///
/// # Example
///
/// ```no_run
/// use jwt_service::{create_redis_client, is_token_revoked};
///
/// # async fn example() -> anyhow::Result<()> {
/// let mut redis_conn = create_redis_client("redis://127.0.0.1:6379").await?;
/// if is_token_revoked(&mut redis_conn, "f47ac10b-...").await? {
///     println!("Token has been revoked!");
/// }
/// # Ok(())
/// # }
/// ```
pub async fn is_token_revoked(redis_conn: &mut ConnectionManager, jti: &str) -> Result<bool> {
    // ---
    let redis_key = format!("blacklist:jti:{}", jti);

    // Check if key exists
    let exists: bool = redis_conn
        .exists(&redis_key)
        .await
        .context("Failed to check token revocation status")?;

    Ok(exists)
}
