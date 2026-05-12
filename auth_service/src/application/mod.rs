use time::Duration;

use crate::{
    domain::{
        AccessTokenId, AuditEventId, AuditEventKind, AuthenticatedUser, Email,
        RefreshTokenFamilyId, RefreshTokenId, SessionId, TokenIntrospection, TokenPair, User,
        UserId,
    },
    error::{AuthError, RepositoryError},
    ports::{
        AuthRepository, Clock, CredentialHasher, NewAccessToken, NewAuditEvent, NewRefreshToken,
        NewSession, NewUser, TokenGenerator,
    },
};

#[derive(Clone, Debug)]
pub struct AuthConfig {
    pub access_token_ttl: Duration,
    pub refresh_token_ttl: Duration,
    pub session_ttl: Duration,
    pub min_password_len: usize,
    pub default_scopes: Vec<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            access_token_ttl: Duration::minutes(15),
            refresh_token_ttl: Duration::days(30),
            session_ttl: Duration::days(30),
            min_password_len: 12,
            default_scopes: vec!["profile:read".to_owned()],
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct AuthContext {
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
}

#[derive(Clone, Debug)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Clone, Debug)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Clone, Debug)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Clone, Debug)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

#[derive(Clone, Debug)]
pub struct AuthService<R, H, T, C> {
    repository: R,
    hasher: H,
    tokens: T,
    clock: C,
    config: AuthConfig,
}

impl<R, H, T, C> AuthService<R, H, T, C>
where
    R: AuthRepository,
    H: CredentialHasher,
    T: TokenGenerator,
    C: Clock,
{
    pub fn new(repository: R, hasher: H, tokens: T, clock: C, config: AuthConfig) -> Self {
        Self {
            repository,
            hasher,
            tokens,
            clock,
            config,
        }
    }

    pub async fn register(&self, request: RegisterRequest) -> Result<User, AuthError> {
        self.validate_password(&request.password)?;

        let email = Email::parse(request.email)?;

        if self.repository.find_user_by_email(&email).await?.is_some() {
            return Err(AuthError::UserAlreadyExists);
        }

        let now = self.clock.now();
        let password_hash = self.hasher.hash_password(&request.password)?;

        let user = self
            .repository
            .create_user(NewUser {
                id: UserId::new(),
                email,
                password_hash,
                created_at: now,
            })
            .await
            .map_err(map_create_user_error)?;

        self.record_audit(Some(user.id), AuditEventKind::UserRegistered, now, None)
            .await?;

        Ok(user)
    }

    pub async fn login(
        &self,
        request: LoginRequest,
        context: AuthContext,
    ) -> Result<AuthenticatedUser, AuthError> {
        let email = Email::parse(request.email)?;
        let now = self.clock.now();
        let Some(user) = self.repository.find_user_by_email(&email).await? else {
            self.record_audit(
                None,
                AuditEventKind::LoginFailed,
                now,
                Some(email.to_string()),
            )
            .await?;
            return Err(AuthError::InvalidCredentials);
        };

        if user.is_disabled() {
            self.record_audit(
                Some(user.id),
                AuditEventKind::LoginFailed,
                now,
                Some("disabled_account".to_owned()),
            )
            .await?;
            return Err(AuthError::AccountDisabled);
        }

        if !self
            .hasher
            .verify_password(&request.password, &user.password_hash)?
        {
            self.record_audit(
                Some(user.id),
                AuditEventKind::LoginFailed,
                now,
                Some("invalid_password".to_owned()),
            )
            .await?;
            return Err(AuthError::InvalidCredentials);
        }

        let (tokens, _) = self
            .issue_token_pair(user.id, None, SessionIssue::New(context), now)
            .await?;

        self.record_audit(Some(user.id), AuditEventKind::LoginSucceeded, now, None)
            .await?;

        Ok(AuthenticatedUser { user, tokens })
    }

    pub async fn refresh(&self, request: RefreshRequest) -> Result<TokenPair, AuthError> {
        let now = self.clock.now();
        let token_hash = self.tokens.hash_token(&request.refresh_token);
        let Some(refresh_token) = self
            .repository
            .find_refresh_token_by_hash(&token_hash)
            .await?
        else {
            return Err(AuthError::InvalidToken);
        };

        if refresh_token.revoked_at.is_some() {
            return Err(AuthError::TokenRevoked);
        }

        if refresh_token.expires_at <= now {
            return Err(AuthError::TokenExpired);
        }

        if refresh_token.rotated_at.is_some() {
            self.repository
                .revoke_refresh_token_family(refresh_token.family_id, now)
                .await?;
            self.record_audit(
                Some(refresh_token.user_id),
                AuditEventKind::RefreshTokenReuseDetected,
                now,
                None,
            )
            .await?;
            return Err(AuthError::TokenReuseDetected);
        }

        let Some(session) = self
            .repository
            .find_session(refresh_token.session_id)
            .await?
        else {
            return Err(AuthError::InvalidToken);
        };

        if !session.is_active_at(now) {
            return Err(AuthError::TokenRevoked);
        }

        let (tokens, new_refresh_token_id) = self
            .issue_token_pair(
                refresh_token.user_id,
                Some(refresh_token.family_id),
                SessionIssue::Existing(session.id),
                now,
            )
            .await?;

        self.repository
            .mark_refresh_token_rotated(refresh_token.id, now, new_refresh_token_id)
            .await?;

        self.record_audit(
            Some(refresh_token.user_id),
            AuditEventKind::TokenRefreshed,
            now,
            None,
        )
        .await?;

        Ok(tokens)
    }

    pub async fn logout(&self, request: LogoutRequest) -> Result<(), AuthError> {
        let now = self.clock.now();
        let token_hash = self.tokens.hash_token(&request.refresh_token);

        let Some(refresh_token) = self
            .repository
            .find_refresh_token_by_hash(&token_hash)
            .await?
        else {
            return Ok(());
        };

        self.repository
            .revoke_session(refresh_token.session_id, now)
            .await?;
        self.repository
            .revoke_refresh_token_family(refresh_token.family_id, now)
            .await?;

        self.record_audit(
            Some(refresh_token.user_id),
            AuditEventKind::Logout,
            now,
            None,
        )
        .await?;

        Ok(())
    }

    pub async fn introspect_access_token(
        &self,
        access_token: &str,
    ) -> Result<TokenIntrospection, AuthError> {
        let now = self.clock.now();
        let token_hash = self.tokens.hash_token(access_token);

        let Some(token) = self
            .repository
            .find_access_token_by_hash(&token_hash)
            .await?
        else {
            return Ok(TokenIntrospection::inactive());
        };

        if !token.is_active_at(now) {
            return Ok(TokenIntrospection::inactive());
        }

        let Some(session) = self.repository.find_session(token.session_id).await? else {
            return Ok(TokenIntrospection::inactive());
        };

        if !session.is_active_at(now) {
            return Ok(TokenIntrospection::inactive());
        }

        Ok(TokenIntrospection {
            active: true,
            user_id: Some(token.user_id),
            session_id: Some(token.session_id),
            expires_at: Some(token.expires_at),
            scopes: token.scopes,
        })
    }

    async fn issue_token_pair(
        &self,
        user_id: UserId,
        existing_family_id: Option<RefreshTokenFamilyId>,
        session_issue: SessionIssue,
        now: time::OffsetDateTime,
    ) -> Result<(TokenPair, RefreshTokenId), AuthError> {
        let refresh_family_id = existing_family_id.unwrap_or_default();
        let access_token_id = AccessTokenId::new();
        let refresh_token_id = RefreshTokenId::new();

        let access_token = self.tokens.generate_token()?;
        let refresh_token = self.tokens.generate_token()?;
        let access_expires_at = now + self.config.access_token_ttl;
        let refresh_expires_at = now + self.config.refresh_token_ttl;

        let session_id = match session_issue {
            SessionIssue::New(context) => {
                let session_id = SessionId::new();
                self.repository
                    .create_session(NewSession {
                        id: session_id,
                        user_id,
                        created_at: now,
                        expires_at: now + self.config.session_ttl,
                        user_agent: context.user_agent,
                        ip_address: context.ip_address,
                    })
                    .await?;
                session_id
            }
            SessionIssue::Existing(session_id) => session_id,
        };

        self.repository
            .create_access_token(NewAccessToken {
                id: access_token_id,
                user_id,
                session_id,
                token_hash: self.tokens.hash_token(&access_token),
                issued_at: now,
                expires_at: access_expires_at,
                scopes: self.config.default_scopes.clone(),
            })
            .await?;

        self.repository
            .create_refresh_token(NewRefreshToken {
                id: refresh_token_id,
                family_id: refresh_family_id,
                user_id,
                session_id,
                token_hash: self.tokens.hash_token(&refresh_token),
                issued_at: now,
                expires_at: refresh_expires_at,
            })
            .await?;

        Ok((
            TokenPair {
                access_token,
                refresh_token,
                token_type: "Bearer".to_owned(),
                access_expires_at,
                refresh_expires_at,
            },
            refresh_token_id,
        ))
    }

    async fn record_audit(
        &self,
        user_id: Option<UserId>,
        kind: AuditEventKind,
        occurred_at: time::OffsetDateTime,
        metadata: Option<String>,
    ) -> Result<(), AuthError> {
        self.repository
            .record_audit_event(NewAuditEvent {
                id: AuditEventId::new(),
                user_id,
                kind,
                occurred_at,
                metadata,
            })
            .await?;

        Ok(())
    }

    fn validate_password(&self, password: &str) -> Result<(), AuthError> {
        if password.trim().len() < self.config.min_password_len {
            return Err(AuthError::WeakPassword);
        }

        if password.chars().all(char::is_whitespace) {
            return Err(AuthError::WeakPassword);
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
enum SessionIssue {
    New(AuthContext),
    Existing(SessionId),
}

fn map_create_user_error(error: RepositoryError) -> AuthError {
    match error {
        RepositoryError::Conflict { entity: "user" } => AuthError::UserAlreadyExists,
        other => AuthError::Repository(other),
    }
}
