// oauth2-server/examples/generate_password_hash.rs

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};

fn main() {
    let password = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "demo123".to_string());

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(hash) => {
            println!("Password: {}", password);
            println!("Hash: {}", hash);
        }
        Err(e) => {
            eprintln!("Error generating hash: {}", e);
            std::process::exit(1);
        }
    }
}
