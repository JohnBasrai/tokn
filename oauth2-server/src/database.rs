// oauth2-server/src/database.rs

use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

// ---

/// Creates a PostgreSQL connection pool.
///
/// Configures a connection pool with a maximum of 5 connections for storing
/// OAuth2 authorization codes, access tokens, and user data.
///
/// # Errors
///
/// Returns an error if the database connection cannot be established.
pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    // ---
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    Ok(pool)
}
