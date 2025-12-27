// jwt-service/src/config.rs

use anyhow::{Context, Result};
use serde::Deserialize;
use std::env;

// ---

/// Application configuration for the JWT service.
///
/// Contains server, Redis, and JWT signing configuration loaded from environment variables.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    // ---
    pub server: ServerConfig,
    pub redis: RedisConfig,
    pub jwt: JwtConfig,
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

/// Redis connection configuration.
///
/// Used for storing refresh tokens and blacklisted JWTs.
#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    // ---
    pub url: String,
}

// ---

/// JWT signing and validation configuration.
///
/// # Security
///
/// - `secret` must be at least 256 bits (32 bytes) for HS256
/// - `access_token_expiry_seconds` should be short (recommended: 900 = 15 minutes)
/// - `refresh_token_expiry_seconds` should be longer (recommended: 604800 = 7 days)
#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    // ---
    /// Secret key for signing JWTs (HS256)
    pub secret: String,
    /// Access token expiry in seconds (default: 900 = 15 minutes)
    pub access_token_expiry_seconds: i64,
    /// Refresh token expiry in seconds (default: 604800 = 7 days)
    pub refresh_token_expiry_seconds: i64,
}

// ---

impl Config {
    // ---
    /// Load configuration from environment variables.
    ///
    /// # Environment Variables
    ///
    /// - `JWT_SERVICE_HOST` (default: "127.0.0.1")
    /// - `JWT_SERVICE_PORT` (default: "8083")
    /// - `REDIS_URL` (default: "redis://127.0.0.1:6379")
    /// - `JWT_SECRET` (required, no default)
    /// - `JWT_ACCESS_TOKEN_EXPIRY_SECONDS` (default: "900")
    /// - `JWT_REFRESH_TOKEN_EXPIRY_SECONDS` (default: "604800")
    ///
    /// # Errors
    ///
    /// Returns error if `JWT_SECRET` is not set or configuration is invalid.
    pub fn from_env() -> Result<Self> {
        // ---
        let server = ServerConfig {
            host: env::var("JWT_SERVICE_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("JWT_SERVICE_PORT")
                .unwrap_or_else(|_| "8083".to_string())
                .parse()
                .context("Invalid JWT_SERVICE_PORT")?,
        };

        let redis = RedisConfig {
            url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
        };

        let jwt = JwtConfig {
            secret: env::var("JWT_SECRET").context("JWT_SECRET environment variable required")?,
            access_token_expiry_seconds: env::var("JWT_ACCESS_TOKEN_EXPIRY_SECONDS")
                .unwrap_or_else(|_| "900".to_string())
                .parse()
                .context("Invalid JWT_ACCESS_TOKEN_EXPIRY_SECONDS")?,
            refresh_token_expiry_seconds: env::var("JWT_REFRESH_TOKEN_EXPIRY_SECONDS")
                .unwrap_or_else(|_| "604800".to_string())
                .parse()
                .context("Invalid JWT_REFRESH_TOKEN_EXPIRY_SECONDS")?,
        };

        // Validate JWT secret length
        if jwt.secret.len() < 32 {
            anyhow::bail!("JWT_SECRET must be at least 32 characters (256 bits) for security");
        }

        Ok(Config {
            server,
            redis,
            jwt,
        })
    }
}
