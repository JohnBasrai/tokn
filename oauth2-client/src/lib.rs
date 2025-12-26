// oauth2-client/src/lib.rs

//! OAuth2 client demonstration application
//!
//! This crate demonstrates OAuth2 authorization code flow by acting as
//! an OAuth2 client that authenticates users via the oauth2-server.

// ---

mod config;
mod handlers;

// ---

pub use config::Config;
pub use handlers::{callback_handler, home_handler, login_handler, profile_handler};
