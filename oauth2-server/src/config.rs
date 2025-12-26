// oauth2-server/src/config.rs

use anyhow::{Context, Result};
use serde::Deserialize;
use std::env;

// ---

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    // ---
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
}

// ---

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    // ---
    pub host: String,
    pub port: u16,
}

// ---

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    // ---
    pub url: String,
}

// ---

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    // ---
    pub url: String,
}

// ---

impl Config {
    // ---
    pub fn from_env() -> Result<Self> {
        // ---
        dotenvy::dotenv().ok();

        // ---
        let server = ServerConfig {
            // ---
            host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8082".to_string())
                .parse()
                .context("SERVER_PORT must be a valid u16")?,
        };

        // ---
        let database = DatabaseConfig {
            // ---
            url: env::var("DATABASE_URL").context("DATABASE_URL must be set")?,
        };

        // ---
        let redis = RedisConfig {
            // ---
            url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
        };

        // ---
        Ok(Self {
            // ---
            server,
            database,
            redis,
        })
    }

    // ---
    pub fn bind_address(&self) -> String {
        // ---
        format!("{}:{}", self.server.host, self.server.port)
    }
}
