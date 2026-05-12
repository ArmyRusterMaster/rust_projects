use std::{future::Future, pin::Pin, sync::Arc};

use crate::{AuthConfig, AuthContext, AuthError, AuthService, adapters::{crypto::{Argon2CredentialHasher, RandomTokenGenerator}, persistence::InMemoryAuthRepository}, application::{LoginRequest, LogoutRequest, RefreshRequest, RegisterRequest}, config::{AppConfig, Backend}, domain::{AuthenticatedUser, PublicUser, TokenIntrospection, TokenPair}, http, ports::SystemClock};

#[cfg(feature = "sqlite")]
use crate::adapters::persistence::SqliteAuthRepository;

pub trait AuthServiceApi: Send + Sync {
    fn register<'a>(&'a self, req: RegisterRequest) -> Pin<Box<dyn Future<Output = Result<PublicUser, AuthError>> + Send + 'a>>;
    fn login<'a>(&'a self, req: LoginRequest, ctx: AuthContext) -> Pin<Box<dyn Future<Output = Result<AuthenticatedUser, AuthError>> + Send + 'a>>;
    fn refresh<'a>(&'a self, req: RefreshRequest) -> Pin<Box<dyn Future<Output = Result<TokenPair, AuthError>> + Send + 'a>>;
    fn logout<'a>(&'a self, req: LogoutRequest) -> Pin<Box<dyn Future<Output = Result<(), AuthError>> + Send + 'a>>;
    fn introspect_access_token<'a>(&'a self, token: &'a str) -> Pin<Box<dyn Future<Output = Result<TokenIntrospection, AuthError>> + Send + 'a>>;
}

impl<R, H, T, C> AuthServiceApi for AuthService<R, H, T, C>
where R: crate::ports::AuthRepository, H: crate::ports::CredentialHasher, T: crate::ports::TokenGenerator, C: crate::ports::Clock {
    fn register<'a>(&'a self, req: RegisterRequest) -> Pin<Box<dyn Future<Output = Result<PublicUser, AuthError>> + Send + 'a>> { Box::pin(async move { self.register(req).await }) }
    fn login<'a>(&'a self, req: LoginRequest, ctx: AuthContext) -> Pin<Box<dyn Future<Output = Result<AuthenticatedUser, AuthError>> + Send + 'a>> { Box::pin(async move { self.login(req, ctx).await }) }
    fn refresh<'a>(&'a self, req: RefreshRequest) -> Pin<Box<dyn Future<Output = Result<TokenPair, AuthError>> + Send + 'a>> { Box::pin(async move { self.refresh(req).await }) }
    fn logout<'a>(&'a self, req: LogoutRequest) -> Pin<Box<dyn Future<Output = Result<(), AuthError>> + Send + 'a>> { Box::pin(async move { self.logout(req).await }) }
    fn introspect_access_token<'a>(&'a self, token: &'a str) -> Pin<Box<dyn Future<Output = Result<TokenIntrospection, AuthError>> + Send + 'a>> { Box::pin(async move { self.introspect_access_token(token).await }) }
}

pub type DynAuthService = dyn AuthServiceApi;

pub struct Server { config: AppConfig }
impl Server {
    pub fn new(config: AppConfig) -> Self { Self { config } }

    pub async fn run(self) -> std::io::Result<()> {
        let service = build_service(&self.config).await.expect("failed to build service");
        let app = http::build_router(service, self.config.request_limit_per_sec);
        let listener = tokio::net::TcpListener::bind(self.config.addr).await?;
        axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()).await
    }
}

async fn build_service(config: &AppConfig) -> Result<Arc<DynAuthService>, String> {
    match &config.backend {
        Backend::Memory => Ok(Arc::new(AuthService::new(InMemoryAuthRepository::new(), Argon2CredentialHasher, RandomTokenGenerator::default(), SystemClock, AuthConfig::default()))),
        Backend::Sqlite { database_url } => {
            #[cfg(feature = "sqlite")]
            {
                let repo = SqliteAuthRepository::connect(database_url).await.map_err(|e| e.to_string())?;
                apply_migrations_sqlite(&repo).await.map_err(|e| e.to_string())?;
                Ok(Arc::new(AuthService::new(repo, Argon2CredentialHasher, RandomTokenGenerator::default(), SystemClock, AuthConfig::default())))
            }
            #[cfg(not(feature = "sqlite"))]
            {
                let _ = database_url;
                Err("binary built without sqlite feature".to_owned())
            }
        }
    }
}

#[cfg(feature = "sqlite")]
async fn apply_migrations_sqlite(repo: &SqliteAuthRepository) -> Result<(), crate::RepositoryError> {
    repo.initialize_schema().await
}

async fn shutdown_signal() {
    let ctrl_c = async { let _ = tokio::signal::ctrl_c().await; };
    #[cfg(unix)]
    let terminate = async {
        let mut signal = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).expect("failed to install SIGTERM handler");
        let _ = signal.recv().await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {}, _ = terminate => {}, }
}
