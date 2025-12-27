// jwt-service/src/handlers/mod.rs

//! JWT service HTTP handlers
//!
//! # Endpoints
//!
//! - `POST /auth/token` - Generate JWT access and refresh tokens
//! - `POST /auth/validate` - Validate JWT signature and expiration
//! - `POST /auth/refresh` - Exchange refresh token for new access token
//! - `POST /auth/revoke` - Revoke (blacklist) a JWT
//! - `GET /protected` - Demo protected endpoint requiring valid JWT

// TODO: Add handler modules as we implement them
// mod token;
// mod validate;
// mod refresh;
// mod revoke;
// mod protected;

// TODO: Re-export handler functions
// pub use token::generate_token_handler;
// pub use validate::validate_token_handler;
// pub use refresh::refresh_token_handler;
// pub use revoke::revoke_token_handler;
// pub use protected::protected_handler;
