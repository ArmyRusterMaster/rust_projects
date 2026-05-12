use async_trait::async_trait;
use time::OffsetDateTime;

use crate::{
    domain::{
        AccessToken, AuditEvent, AuditEventKind, Email, RefreshToken, RefreshTokenFamilyId,
        RefreshTokenId, Session, SessionId, User, UserId,
    },
    error::RepositoryError,
};

#[derive(Clone, Debug)]
pub struct NewUser {
    pub id: UserId,
    pub email: Email,
    pub password_hash: String,
    pub created_at: OffsetDateTime,
}

#[derive(Clone, Debug)]
pub struct NewSession {
    pub id: SessionId,
    pub user_id: UserId,
    pub created_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
}

#[derive(Clone, Debug)]
pub struct NewAccessToken {
    pub id: crate::domain::AccessTokenId,
    pub user_id: UserId,
    pub session_id: SessionId,
    pub token_hash: String,
    pub issued_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
    pub scopes: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct NewRefreshToken {
    pub id: RefreshTokenId,
    pub family_id: RefreshTokenFamilyId,
    pub user_id: UserId,
    pub session_id: SessionId,
    pub token_hash: String,
    pub issued_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
}

#[derive(Clone, Debug)]
pub struct NewAuditEvent {
    pub id: crate::domain::AuditEventId,
    pub user_id: Option<UserId>,
    pub kind: AuditEventKind,
    pub occurred_at: OffsetDateTime,
    pub metadata: Option<String>,
}

#[async_trait]
pub trait AuthRepository: Clone + Send + Sync + 'static {
    async fn create_user(&self, user: NewUser) -> Result<User, RepositoryError>;

    async fn find_user_by_email(&self, email: &Email) -> Result<Option<User>, RepositoryError>;

    async fn create_session(&self, session: NewSession) -> Result<Session, RepositoryError>;

    async fn find_session(&self, session_id: SessionId)
    -> Result<Option<Session>, RepositoryError>;

    async fn revoke_session(
        &self,
        session_id: SessionId,
        revoked_at: OffsetDateTime,
    ) -> Result<(), RepositoryError>;

    async fn create_access_token(
        &self,
        token: NewAccessToken,
    ) -> Result<AccessToken, RepositoryError>;

    async fn find_access_token_by_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<AccessToken>, RepositoryError>;

    async fn create_refresh_token(
        &self,
        token: NewRefreshToken,
    ) -> Result<RefreshToken, RepositoryError>;

    async fn find_refresh_token_by_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<RefreshToken>, RepositoryError>;

    async fn mark_refresh_token_rotated(
        &self,
        token_id: RefreshTokenId,
        rotated_at: OffsetDateTime,
        replaced_by: RefreshTokenId,
    ) -> Result<(), RepositoryError>;

    async fn revoke_refresh_token_family(
        &self,
        family_id: RefreshTokenFamilyId,
        revoked_at: OffsetDateTime,
    ) -> Result<usize, RepositoryError>;

    async fn record_audit_event(&self, event: NewAuditEvent)
    -> Result<AuditEvent, RepositoryError>;
}
