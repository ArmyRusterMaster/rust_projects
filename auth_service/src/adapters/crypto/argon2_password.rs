use argon2::{
    Argon2,
    password_hash::{
        PasswordHash, PasswordHasher as ArgonPasswordHasher, PasswordVerifier, SaltString,
    },
};
use rand_core::OsRng;
use secrecy::{ExposeSecret, SecretString};

use crate::{error::AuthError, ports::CredentialHasher};

#[derive(Clone, Debug, Default)]
pub struct Argon2CredentialHasher;

impl CredentialHasher for Argon2CredentialHasher {
    fn hash_password(&self, password: &str) -> Result<SecretString, AuthError> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|error| AuthError::PasswordHashing(error.to_string()))?
            .to_string();

        Ok(SecretString::new(password_hash.into()))
    }

    fn verify_password(&self, password: &str, password_hash: &SecretString) -> Result<bool, AuthError> {
        let parsed_hash = PasswordHash::new(password_hash.expose_secret())
            .map_err(|error| AuthError::PasswordHashing(error.to_string()))?;

        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}
