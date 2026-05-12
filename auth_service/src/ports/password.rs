use crate::error::AuthError;

pub trait CredentialHasher: Clone + Send + Sync + 'static {
    fn hash_password(&self, password: &str) -> Result<String, AuthError>;

    fn verify_password(&self, password: &str, password_hash: &str) -> Result<bool, AuthError>;
}
