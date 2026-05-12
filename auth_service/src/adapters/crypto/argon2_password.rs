use argon2::{
    Argon2,
    password_hash::{
        PasswordHash, PasswordHasher as ArgonPasswordHasher, PasswordVerifier, SaltString,
    },
};
use rand_core::OsRng;

use crate::{error::AuthError, ports::CredentialHasher};

#[derive(Clone, Debug, Default)]
pub struct Argon2CredentialHasher;

impl CredentialHasher for Argon2CredentialHasher {
    fn hash_password(&self, password: &str) -> Result<String, AuthError> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|error| AuthError::PasswordHashing(error.to_string()))?
            .to_string();

        Ok(password_hash)
    }

    fn verify_password(&self, password: &str, password_hash: &str) -> Result<bool, AuthError> {
        let parsed_hash = PasswordHash::new(password_hash)
            .map_err(|error| AuthError::PasswordHashing(error.to_string()))?;

        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}
