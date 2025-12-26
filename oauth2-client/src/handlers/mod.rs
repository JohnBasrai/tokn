// oauth2-client/src/handlers/mod.rs

//! HTTP request handlers for oauth2-client

// ---

mod callback;
mod home;
mod login;
mod profile;

// ---

pub use callback::callback_handler;
pub use home::home_handler;
pub use login::login_handler;
pub use profile::profile_handler;
