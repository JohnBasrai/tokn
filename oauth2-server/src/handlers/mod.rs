// oauth2-server/src/handlers/mod.rs

//! HTTP request handlers for oauth2-server

// ---

mod authorize;
mod authorize_post;
mod token;
mod userinfo;

// ---
pub use authorize::authorize_handler;
pub use authorize_post::authorize_post_handler;
pub use token::token_handler;
pub use userinfo::userinfo_handler;
