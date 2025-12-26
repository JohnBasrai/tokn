// oauth2-client/src/config.rs

use anyhow::{Context, Result};
use serde::Deserialize;
use std::env;

// ---

/// Application configuration for the OAuth2 client.
///
/// Contains server, Redis, and OAuth2 provider settings loaded from environment variables.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    // ---
    pub server: ServerConfig,
    pub redis: RedisConfig,
    pub oauth2: OAuth2Config,
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
/// Used for session storage and CSRF token validation.
#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    // ---
    pub url: String,
}

// ---

/// OAuth2 provider configuration.
///
/// Contains OAuth2 client credentials and endpoint URLs for the authorization server.
#[derive(Debug, Clone, Deserialize)]
pub struct OAuth2Config {
    // ---
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub authorize_url: String,
    pub token_url: String,
    pub userinfo_url: String,
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
    /// - Required environment variables are missing (OAUTH2_CLIENT_ID, OAUTH2_CLIENT_SECRET, DATABASE_URL)
    /// - Port values cannot be parsed as u16
    pub fn from_env() -> Result<Self> {
        // ---
        dotenvy::dotenv().ok();

        // ---
        let server = ServerConfig {
            host: env::var("CLIENT_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("CLIENT_PORT")
                .unwrap_or_else(|_| "8081".to_string())
                .parse()
                .context("CLIENT_PORT must be a valid u16")?,
        };

        // ---
        let redis = RedisConfig {
            url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
        };

        // ---
        let oauth2 = OAuth2Config {
            client_id: env::var("OAUTH2_CLIENT_ID").context("OAUTH2_CLIENT_ID must be set")?,
            client_secret: env::var("OAUTH2_CLIENT_SECRET")
                .context("OAUTH2_CLIENT_SECRET must be set")?,
            redirect_uri: env::var("OAUTH2_REDIRECT_URI")
                .unwrap_or_else(|_| "http://127.0.0.1:8081/callback".to_string()),
            authorize_url: env::var("OAUTH2_AUTHORIZE_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8082/oauth/authorize".to_string()),
            token_url: env::var("OAUTH2_TOKEN_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8082/oauth/token".to_string()),
            userinfo_url: env::var("OAUTH2_USERINFO_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8082/oauth/userinfo".to_string()),
        };

        // ---
        Ok(Self {
            server,
            redis,
            oauth2,
        })
    }

    // ---
    /// Returns the server bind address in "host:port" format.
    pub fn bind_address(&self) -> String {
        // ---
        format!("{}:{}", self.server.host, self.server.port)
    }
}
