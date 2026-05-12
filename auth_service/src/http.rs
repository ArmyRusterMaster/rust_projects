use std::{collections::HashMap, net::IpAddr, sync::{Arc, Mutex}, time::{Duration, Instant}};

use axum::{
    Json, Router,
    extract::{ConnectInfo, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tower::limit::GlobalConcurrencyLimitLayer;
use tower_http::{
    cors::{Any, CorsLayer},
    request_id::{MakeRequestUuid, SetRequestIdLayer},
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};

use crate::{
    AuthContext, AuthError,
    application::{
        LoginRequest as AppLoginRequest, LogoutRequest as AppLogoutRequest,
        RefreshRequest as AppRefreshRequest, RegisterRequest as AppRegisterRequest,
    },
    domain::{AuthenticatedUser, PublicUser, TokenIntrospection, TokenPair},
    policy::PolicyInput,
    server::{DynAuthService, SharedPolicyEngine},
};

#[derive(Clone)]
pub struct AppState {
    pub service: Arc<DynAuthService>,
    pub policy: SharedPolicyEngine,
    pub abuse: Arc<Mutex<AbuseGuard>>,
}

pub fn build_router(service: Arc<DynAuthService>, policy: SharedPolicyEngine, max_inflight: usize) -> Router {
    let state = AppState { service, policy, abuse: Arc::new(Mutex::new(AbuseGuard::default())) };

    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
        .route("/introspect", post(introspect))
        .route("/authorize/check", post(authorize_check))
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .layer(GlobalConcurrencyLimitLayer::new(max_inflight))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::HeaderName::from_static("x-content-type-options"),
            axum::http::HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::HeaderName::from_static("x-frame-options"),
            axum::http::HeaderValue::from_static("DENY"),
        ))
        .layer(CorsLayer::new().allow_methods(Any).allow_origin(Any))
        .layer(TraceLayer::new_for_http())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .with_state(state)
}

#[derive(Default)]
struct AbuseGuard {
    by_email: HashMap<String, FailedState>,
    by_ip: HashMap<IpAddr, FailedState>,
}

#[derive(Clone)]
struct FailedState { fails: u32, blocked_until: Option<Instant> }
impl Default for FailedState { fn default() -> Self { Self { fails: 0, blocked_until: None } } }

impl AbuseGuard {
    fn is_blocked(&mut self, email: &str, ip: IpAddr) -> bool {
        Self::is_key_blocked(self.by_email.get_mut(email)) || Self::is_key_blocked(self.by_ip.get_mut(&ip))
    }
    fn on_fail(&mut self, email: &str, ip: IpAddr) {
        Self::mark_fail(self.by_email.entry(email.to_owned()).or_default());
        Self::mark_fail(self.by_ip.entry(ip).or_default());
    }
    fn on_success(&mut self, email: &str, ip: IpAddr) {
        self.by_email.remove(email);
        self.by_ip.remove(&ip);
    }
    fn is_key_blocked(state: Option<&mut FailedState>) -> bool {
        if let Some(s) = state {
            if let Some(until) = s.blocked_until {
                if Instant::now() < until { return true; }
                s.blocked_until = None;
            }
        }
        false
    }
    fn mark_fail(state: &mut FailedState) {
        state.fails += 1;
        if state.fails >= 5 {
            let secs = (state.fails.min(10) - 4) * 5;
            state.blocked_until = Some(Instant::now() + Duration::from_secs(secs as u64));
        }
    }
}

#[derive(Debug, Deserialize)]
struct RegisterRequest { email: String, password: String }
#[derive(Debug, Deserialize)]
struct LoginRequest { email: String, password: String }
#[derive(Debug, Deserialize)]
struct RefreshRequest { refresh_token: String }
#[derive(Debug, Deserialize)]
struct LogoutRequest { refresh_token: String }
#[derive(Debug, Deserialize)]
struct AccessTokenRequest { access_token: String }
#[derive(Debug, Deserialize)]
struct AuthorizeRequest { role: String, action: String, resource_owner: String, subject_id: String }

#[derive(Debug, Serialize)]
struct ErrorBody { error: &'static str }
#[derive(Debug, Serialize)]
struct AuthorizeResponse { allow: bool }

struct ApiError(AuthError);
impl From<AuthError> for ApiError { fn from(value: AuthError) -> Self { Self(value) } }

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code) = match self.0 {
            AuthError::InvalidEmail | AuthError::WeakPassword => (StatusCode::BAD_REQUEST, "invalid_request"),
            AuthError::UserAlreadyExists => (StatusCode::CONFLICT, "conflict"),
            AuthError::InvalidCredentials | AuthError::InvalidToken | AuthError::TokenExpired | AuthError::TokenRevoked | AuthError::TokenReuseDetected => (StatusCode::UNAUTHORIZED, "unauthorized"),
            AuthError::AccountDisabled => (StatusCode::FORBIDDEN, "forbidden"),
            AuthError::PasswordHashing(_) | AuthError::TokenGeneration(_) | AuthError::Repository(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
        };
        (status, Json(ErrorBody { error: code })).into_response()
    }
}

async fn register(State(state): State<AppState>, Json(request): Json<RegisterRequest>) -> Result<(StatusCode, Json<PublicUser>), ApiError> {
    let user = state.service.register(AppRegisterRequest { email: request.email, password: request.password }).await?;
    Ok((StatusCode::CREATED, Json(user)))
}

async fn login(
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<AuthenticatedUser>, ApiError> {
    {
        let mut guard = state.abuse.lock().expect("abuse mutex poisoned");
        if guard.is_blocked(&request.email, addr.ip()) {
            return Err(ApiError(AuthError::InvalidCredentials));
        }
    }

    let email_for_guard = request.email.clone();
    let result = state.service.login(AppLoginRequest { email: request.email, password: request.password }, AuthContext::default()).await;
    match result {
        Ok(auth) => {
            let mut guard = state.abuse.lock().expect("abuse mutex poisoned");
            guard.on_success(&email_for_guard, addr.ip());
            Ok(Json(auth))
        }
        Err(e) => {
            let mut guard = state.abuse.lock().expect("abuse mutex poisoned");
            guard.on_fail(&email_for_guard, addr.ip());
            Err(ApiError(e))
        }
    }
}

async fn refresh(State(state): State<AppState>, Json(request): Json<RefreshRequest>) -> Result<Json<TokenPair>, ApiError> {
    Ok(Json(state.service.refresh(AppRefreshRequest { refresh_token: request.refresh_token }).await?))
}
async fn logout(State(state): State<AppState>, Json(request): Json<LogoutRequest>) -> Result<StatusCode, ApiError> {
    state.service.logout(AppLogoutRequest { refresh_token: request.refresh_token }).await?;
    Ok(StatusCode::NO_CONTENT)
}
async fn introspect(State(state): State<AppState>, Json(request): Json<AccessTokenRequest>) -> Result<Json<TokenIntrospection>, ApiError> {
    Ok(Json(state.service.introspect_access_token(&request.access_token).await?))
}

async fn authorize_check(State(state): State<AppState>, Json(request): Json<AuthorizeRequest>) -> Result<Json<AuthorizeResponse>, ApiError> {
    let engine = state.policy.read().map_err(|e| ApiError(AuthError::Repository(crate::RepositoryError::Internal(e.to_string()))))?;
    let allow = engine.evaluate(&PolicyInput {
        role: request.role,
        action: request.action,
        resource_owner: request.resource_owner,
        subject_id: request.subject_id,
    }).map_err(AuthError::Repository)?;
    Ok(Json(AuthorizeResponse { allow }))
}

async fn healthz() -> StatusCode { StatusCode::OK }
async fn readyz(State(state): State<AppState>) -> StatusCode {
    match state.service.introspect_access_token("readiness-probe-token").await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}
