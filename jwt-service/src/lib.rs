// jwt-service/src/lib.rs

//! JWT token service library
//!
//! Provides JWT generation, validation, refresh, and revocation functionality.

mod claims;
mod config;
mod handlers;
mod token;

// ---

pub use claims::Claims;
pub use config::Config;
pub use handlers::generate_token_handler;
pub use token::generate_token;
