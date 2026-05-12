use std::{fmt, str::FromStr};

use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::AuthError;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct UserId(Uuid);

impl UserId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SessionId(Uuid);

impl SessionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct AccessTokenId(Uuid);

impl AccessTokenId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for AccessTokenId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for AccessTokenId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct RefreshTokenId(Uuid);

impl RefreshTokenId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for RefreshTokenId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for RefreshTokenId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct RefreshTokenFamilyId(Uuid);

impl RefreshTokenFamilyId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for RefreshTokenFamilyId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for RefreshTokenFamilyId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct AuditEventId(Uuid);

impl AuditEventId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for AuditEventId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for AuditEventId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Email(String);

impl Email {
    pub fn parse(input: impl AsRef<str>) -> Result<Self, AuthError> {
        let normalized = input.as_ref().trim().to_ascii_lowercase();

        if normalized.len() > 254
            || normalized.is_empty()
            || normalized.contains(char::is_whitespace)
            || normalized.matches('@').count() != 1
        {
            return Err(AuthError::InvalidEmail);
        }

        let mut parts = normalized.split('@');
        let local = parts.next().unwrap_or_default();
        let domain = parts.next().unwrap_or_default();

        if local.is_empty() || domain.is_empty() || !domain.contains('.') {
            return Err(AuthError::InvalidEmail);
        }

        Ok(Self(normalized))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Email {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for Email {
    type Err = AuthError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

#[derive(Clone, Debug)]
pub struct UserRecord {
    pub id: UserId,
    pub email: Email,
    pub password_hash: SecretString,
    pub created_at: OffsetDateTime,
    pub disabled_at: Option<OffsetDateTime>,
}

impl UserRecord {
    pub fn is_disabled(&self) -> bool {
        self.disabled_at.is_some()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicUser {
    pub id: UserId,
    pub email: Email,
    pub created_at: OffsetDateTime,
    pub disabled_at: Option<OffsetDateTime>,
}

impl From<&UserRecord> for PublicUser {
    fn from(value: &UserRecord) -> Self {
        Self {
            id: value.id,
            email: value.email.clone(),
            created_at: value.created_at,
            disabled_at: value.disabled_at,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub user_id: UserId,
    pub created_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
    pub revoked_at: Option<OffsetDateTime>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
}

impl Session {
    pub fn is_active_at(&self, now: OffsetDateTime) -> bool {
        self.revoked_at.is_none() && self.expires_at > now
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AccessToken {
    pub id: AccessTokenId,
    pub user_id: UserId,
    pub session_id: SessionId,
    pub token_hash: String,
    pub issued_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
    pub revoked_at: Option<OffsetDateTime>,
    pub scopes: Vec<String>,
}

impl AccessToken {
    pub fn is_active_at(&self, now: OffsetDateTime) -> bool {
        self.revoked_at.is_none() && self.expires_at > now
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RefreshToken {
    pub id: RefreshTokenId,
    pub family_id: RefreshTokenFamilyId,
    pub user_id: UserId,
    pub session_id: SessionId,
    pub token_hash: String,
    pub issued_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
    pub rotated_at: Option<OffsetDateTime>,
    pub revoked_at: Option<OffsetDateTime>,
    pub replaced_by: Option<RefreshTokenId>,
}

impl RefreshToken {
    pub fn is_active_at(&self, now: OffsetDateTime) -> bool {
        self.rotated_at.is_none() && self.revoked_at.is_none() && self.expires_at > now
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: AuditEventId,
    pub user_id: Option<UserId>,
    pub kind: AuditEventKind,
    pub occurred_at: OffsetDateTime,
    pub metadata: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum AuditEventKind {
    UserRegistered,
    LoginSucceeded,
    LoginFailed,
    TokenRefreshed,
    RefreshTokenReuseDetected,
    Logout,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub access_expires_at: OffsetDateTime,
    pub refresh_expires_at: OffsetDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub user: PublicUser,
    pub tokens: TokenPair,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TokenIntrospection {
    pub active: bool,
    pub user_id: Option<UserId>,
    pub session_id: Option<SessionId>,
    pub expires_at: Option<OffsetDateTime>,
    pub scopes: Vec<String>,
}

impl TokenIntrospection {
    pub fn inactive() -> Self {
        Self {
            active: false,
            user_id: None,
            session_id: None,
            expires_at: None,
            scopes: Vec::new(),
        }
    }
}
