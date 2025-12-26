// oauth2-server/src/lib.rs

//! OAuth2 authorization server implementation
//!
//! This crate implements an OAuth2 authorization server that issues
//! access tokens following the authorization code flow.

// ---

mod config;
mod database;
mod handlers;

// ---

pub use config::Config;
pub use database::create_pool;
pub use handlers::{
    //
    authorize_handler,
    authorize_post_handler,
    token_handler,
    userinfo_handler,
};
