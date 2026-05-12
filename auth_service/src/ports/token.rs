use crate::error::AuthError;

pub trait TokenGenerator: Clone + Send + Sync + 'static {
    fn generate_token(&self) -> Result<String, AuthError>;

    fn hash_token(&self, token: &str) -> String;
}
