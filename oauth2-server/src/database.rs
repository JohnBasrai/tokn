// oauth2-server/src/database.rs

use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

// ---

pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    // ---
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    Ok(pool)
}
