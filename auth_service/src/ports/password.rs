use secrecy::SecretString;

use crate::error::AuthError;

pub trait CredentialHasher: Clone + Send + Sync + 'static {
    fn hash_password(&self, password: &str) -> Result<SecretString, AuthError>;

    fn verify_password(&self, password: &str, password_hash: &SecretString) -> Result<bool, AuthError>;
}
