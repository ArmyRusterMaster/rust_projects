use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
};
use serde::{Deserialize, Serialize};

use crate::{
    AuthConfig, AuthContext, AuthError, AuthService,
    adapters::{
        crypto::{Argon2CredentialHasher, RandomTokenGenerator},
        persistence::InMemoryAuthRepository,
    },
    application::{
        LoginRequest as AppLoginRequest, LogoutRequest as AppLogoutRequest,
        RefreshRequest as AppRefreshRequest, RegisterRequest as AppRegisterRequest,
    },
    domain::{AuthenticatedUser, PublicUser, TokenIntrospection, TokenPair},
    ports::SystemClock,
};

type AppService =
    AuthService<InMemoryAuthRepository, Argon2CredentialHasher, RandomTokenGenerator, SystemClock>;

#[derive(Clone)]
pub struct AppState {
    pub service: Arc<AppService>,
}

pub fn build_router() -> Router {
    let service = AuthService::new(
        InMemoryAuthRepository::new(),
        Argon2CredentialHasher,
        RandomTokenGenerator::default(),
        SystemClock,
        AuthConfig::default(),
    );

    let state = AppState {
        service: Arc::new(service),
    };

    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
        .route("/introspect", post(introspect))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
struct RegisterRequest {
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct RefreshRequest {
    refresh_token: String,
}

#[derive(Debug, Deserialize)]
struct LogoutRequest {
    refresh_token: String,
}

#[derive(Debug, Deserialize)]
struct AccessTokenRequest {
    access_token: String,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

struct ApiError(AuthError);

impl From<AuthError> for ApiError {
    fn from(value: AuthError) -> Self {
        Self(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.0 {
            AuthError::InvalidEmail | AuthError::WeakPassword => StatusCode::BAD_REQUEST,
            AuthError::UserAlreadyExists => StatusCode::CONFLICT,
            AuthError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            AuthError::AccountDisabled => StatusCode::FORBIDDEN,
            AuthError::InvalidToken | AuthError::TokenExpired | AuthError::TokenRevoked => {
                StatusCode::UNAUTHORIZED
            }
            AuthError::TokenReuseDetected => StatusCode::CONFLICT,
            AuthError::PasswordHashing(_) | AuthError::TokenGeneration(_) | AuthError::Repository(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        (status, Json(ErrorBody { error: self.0.to_string() })).into_response()
    }
}

async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<PublicUser>), ApiError> {
    let user = state
        .service
        .register(AppRegisterRequest {
            email: request.email,
            password: request.password,
        })
        .await?;
    Ok((StatusCode::CREATED, Json(user)))
}

async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<AuthenticatedUser>, ApiError> {
    let auth = state
        .service
        .login(
            AppLoginRequest {
                email: request.email,
                password: request.password,
            },
            AuthContext::default(),
        )
        .await?;
    Ok(Json(auth))
}

async fn refresh(
    State(state): State<AppState>,
    Json(request): Json<RefreshRequest>,
) -> Result<Json<TokenPair>, ApiError> {
    Ok(Json(
        state
            .service
            .refresh(AppRefreshRequest {
                refresh_token: request.refresh_token,
            })
            .await?,
    ))
}

async fn logout(
    State(state): State<AppState>,
    Json(request): Json<LogoutRequest>,
) -> Result<StatusCode, ApiError> {
    state
        .service
        .logout(AppLogoutRequest {
            refresh_token: request.refresh_token,
        })
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn introspect(
    State(state): State<AppState>,
    Json(request): Json<AccessTokenRequest>,
) -> Result<Json<TokenIntrospection>, ApiError> {
    let result = state
        .service
        .introspect_access_token(&request.access_token)
        .await?;
    Ok(Json(result))
}
