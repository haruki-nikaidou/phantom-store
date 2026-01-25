use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{PasswordHash, PasswordHasher, PasswordVerifier};

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let config = argon2::Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    config
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<(), argon2::password_hash::Error> {
    let config = argon2::Argon2::default();
    let parsed_hash = PasswordHash::new(hash)?;
    config.verify_password(password.as_bytes(), &parsed_hash)
}
