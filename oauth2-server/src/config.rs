// oauth2-server/src/config.rs

use anyhow::{Context, Result};
use serde::Deserialize;
use std::env;

// ---

/// Application configuration for the OAuth2 authorization server.
///
/// Contains server, database, and Redis settings loaded from environment variables.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    // ---
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
}

// ---

/// HTTP server configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    // ---
    pub host: String,
    pub port: u16,
}

// ---

/// PostgreSQL database configuration.
///
/// Used for storing OAuth2 clients, authorization codes, and access tokens.
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    // ---
    pub url: String,
}

// ---

/// Redis connection configuration.
///
/// Used for session storage and token caching.
#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    // ---
    pub url: String,
}

// ---

impl Config {
    // ---
    /// Loads configuration from environment variables.
    ///
    /// Reads configuration from .env file if present, then from environment variables.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Required environment variables are missing (DATABASE_URL)
    /// - Port values cannot be parsed as u16
    pub fn from_env() -> Result<Self> {
        // ---
        dotenvy::dotenv().ok();

        // ---
        let server = ServerConfig {
            host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8082".to_string())
                .parse()
                .context("SERVER_PORT must be a valid u16")?,
        };

        // ---
        let database = DatabaseConfig {
            url: env::var("DATABASE_URL").context("DATABASE_URL must be set")?,
        };

        // ---
        let redis = RedisConfig {
            url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
        };

        // ---
        Ok(Self {
            server,
            database,
            redis,
        })
    }

    // ---
    /// Returns the server bind address in "host:port" format.
    pub fn bind_address(&self) -> String {
        // ---
        format!("{}:{}", self.server.host, self.server.port)
    }
}
