// jwt-service/src/lib.rs

//! JWT token service library
//!
//! Provides JWT generation, validation, refresh, and revocation functionality.

mod config;
mod handlers;

// ---

pub use config::Config;
