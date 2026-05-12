use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("invalid email address")]
    InvalidEmail,
    #[error("password does not satisfy the configured policy")]
    WeakPassword,
    #[error("user already exists")]
    UserAlreadyExists,
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("account is disabled")]
    AccountDisabled,
    #[error("token is invalid")]
    InvalidToken,
    #[error("token is expired")]
    TokenExpired,
    #[error("token is revoked")]
    TokenRevoked,
    #[error("refresh token reuse was detected")]
    TokenReuseDetected,
    #[error("password hashing failed: {0}")]
    PasswordHashing(String),
    #[error("token generation failed: {0}")]
    TokenGeneration(String),
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("{entity} already exists")]
    Conflict { entity: &'static str },
    #[error("{entity} was not found")]
    NotFound { entity: &'static str },
    #[error("{adapter} adapter does not implement {operation}")]
    UnsupportedAdapter {
        adapter: &'static str,
        operation: &'static str,
    },
    #[error("repository failure: {0}")]
    Internal(String),
}

#[derive(Debug, Error)]
pub enum StartupError {
    #[error("invalid configuration: {0}")]
    Config(String),
    #[error("failed to initialize backend: {0}")]
    BackendInit(String),
    #[error("migration failed: {0}")]
    Migration(String),
    #[error("server I/O error: {0}")]
    Io(#[from] std::io::Error),
}
