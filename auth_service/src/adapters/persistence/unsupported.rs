use async_trait::async_trait;
use time::OffsetDateTime;

use crate::{
    domain::{
        AccessToken, AuditEvent, Email, RefreshToken, RefreshTokenFamilyId, RefreshTokenId,
        Session, SessionId, UserRecord,
    },
    error::RepositoryError,
    ports::{AuthRepository, NewAccessToken, NewAuditEvent, NewRefreshToken, NewSession, NewUser},
};

#[derive(Clone, Debug)]
pub struct UnsupportedAuthRepository {
    adapter: &'static str,
}

impl UnsupportedAuthRepository {
    pub fn new(adapter: &'static str) -> Self {
        Self { adapter }
    }

    fn unsupported<T>(&self, operation: &'static str) -> Result<T, RepositoryError> {
        Err(RepositoryError::UnsupportedAdapter {
            adapter: self.adapter,
            operation,
        })
    }
}

#[async_trait]
impl AuthRepository for UnsupportedAuthRepository {
    async fn create_user(&self, _user: NewUser) -> Result<UserRecord, RepositoryError> {
        self.unsupported("create_user")
    }

    async fn find_user_by_email(&self, _email: &Email) -> Result<Option<UserRecord>, RepositoryError> {
        self.unsupported("find_user_by_email")
    }

    async fn create_session(&self, _session: NewSession) -> Result<Session, RepositoryError> {
        self.unsupported("create_session")
    }

    async fn find_session(
        &self,
        _session_id: SessionId,
    ) -> Result<Option<Session>, RepositoryError> {
        self.unsupported("find_session")
    }

    async fn revoke_session(
        &self,
        _session_id: SessionId,
        _revoked_at: OffsetDateTime,
    ) -> Result<(), RepositoryError> {
        self.unsupported("revoke_session")
    }

    async fn create_access_token(
        &self,
        _token: NewAccessToken,
    ) -> Result<AccessToken, RepositoryError> {
        self.unsupported("create_access_token")
    }

    async fn find_access_token_by_hash(
        &self,
        _token_hash: &str,
    ) -> Result<Option<AccessToken>, RepositoryError> {
        self.unsupported("find_access_token_by_hash")
    }

    async fn create_refresh_token(
        &self,
        _token: NewRefreshToken,
    ) -> Result<RefreshToken, RepositoryError> {
        self.unsupported("create_refresh_token")
    }

    async fn find_refresh_token_by_hash(
        &self,
        _token_hash: &str,
    ) -> Result<Option<RefreshToken>, RepositoryError> {
        self.unsupported("find_refresh_token_by_hash")
    }

    async fn rotate_refresh_token(
        &self,
        _token_id: RefreshTokenId,
        _rotated_at: OffsetDateTime,
        _new_token: NewRefreshToken,
    ) -> Result<(), RepositoryError> {
        self.unsupported("rotate_refresh_token")
    }

    async fn revoke_refresh_token_family(
        &self,
        _family_id: RefreshTokenFamilyId,
        _revoked_at: OffsetDateTime,
    ) -> Result<usize, RepositoryError> {
        self.unsupported("revoke_refresh_token_family")
    }

    async fn record_audit_event(
        &self,
        _event: NewAuditEvent,
    ) -> Result<AuditEvent, RepositoryError> {
        self.unsupported("record_audit_event")
    }
}

#[cfg(any(feature = "postgres", feature = "surrealdb"))]
macro_rules! unsupported_repository_adapter {
    ($type_name:ident, $adapter:literal) => {
        #[derive(Clone, Debug)]
        pub struct $type_name {
            database_url: String,
            delegate: $crate::adapters::persistence::unsupported::UnsupportedAuthRepository,
        }

        impl $type_name {
            pub fn new(database_url: impl Into<String>) -> Self {
                Self {
                    database_url: database_url.into(),
                    delegate: $crate::adapters::persistence::unsupported::UnsupportedAuthRepository::new(
                        $adapter,
                    ),
                }
            }

            pub fn database_url(&self) -> &str {
                &self.database_url
            }
        }

        #[async_trait::async_trait]
        impl $crate::ports::AuthRepository for $type_name {
            async fn create_user(
                &self,
                user: $crate::ports::NewUser,
            ) -> Result<$crate::domain::UserRecord, $crate::error::RepositoryError> {
                <$crate::adapters::persistence::unsupported::UnsupportedAuthRepository as $crate::ports::AuthRepository>::create_user(&self.delegate, user).await
            }

            async fn find_user_by_email(
                &self,
                email: &$crate::domain::Email,
            ) -> Result<Option<$crate::domain::UserRecord>, $crate::error::RepositoryError> {
                <$crate::adapters::persistence::unsupported::UnsupportedAuthRepository as $crate::ports::AuthRepository>::find_user_by_email(&self.delegate, email).await
            }

            async fn create_session(
                &self,
                session: $crate::ports::NewSession,
            ) -> Result<$crate::domain::Session, $crate::error::RepositoryError> {
                <$crate::adapters::persistence::unsupported::UnsupportedAuthRepository as $crate::ports::AuthRepository>::create_session(&self.delegate, session).await
            }

            async fn find_session(
                &self,
                session_id: $crate::domain::SessionId,
            ) -> Result<Option<$crate::domain::Session>, $crate::error::RepositoryError> {
                <$crate::adapters::persistence::unsupported::UnsupportedAuthRepository as $crate::ports::AuthRepository>::find_session(&self.delegate, session_id).await
            }

            async fn revoke_session(
                &self,
                session_id: $crate::domain::SessionId,
                revoked_at: time::OffsetDateTime,
            ) -> Result<(), $crate::error::RepositoryError> {
                <$crate::adapters::persistence::unsupported::UnsupportedAuthRepository as $crate::ports::AuthRepository>::revoke_session(&self.delegate, session_id, revoked_at).await
            }

            async fn create_access_token(
                &self,
                token: $crate::ports::NewAccessToken,
            ) -> Result<$crate::domain::AccessToken, $crate::error::RepositoryError> {
                <$crate::adapters::persistence::unsupported::UnsupportedAuthRepository as $crate::ports::AuthRepository>::create_access_token(&self.delegate, token).await
            }

            async fn find_access_token_by_hash(
                &self,
                token_hash: &str,
            ) -> Result<Option<$crate::domain::AccessToken>, $crate::error::RepositoryError> {
                <$crate::adapters::persistence::unsupported::UnsupportedAuthRepository as $crate::ports::AuthRepository>::find_access_token_by_hash(&self.delegate, token_hash).await
            }

            async fn create_refresh_token(
                &self,
                token: $crate::ports::NewRefreshToken,
            ) -> Result<$crate::domain::RefreshToken, $crate::error::RepositoryError> {
                <$crate::adapters::persistence::unsupported::UnsupportedAuthRepository as $crate::ports::AuthRepository>::create_refresh_token(&self.delegate, token).await
            }

            async fn find_refresh_token_by_hash(
                &self,
                token_hash: &str,
            ) -> Result<Option<$crate::domain::RefreshToken>, $crate::error::RepositoryError> {
                <$crate::adapters::persistence::unsupported::UnsupportedAuthRepository as $crate::ports::AuthRepository>::find_refresh_token_by_hash(&self.delegate, token_hash).await
            }

            async fn rotate_refresh_token(
                &self,
                token_id: $crate::domain::RefreshTokenId,
                rotated_at: time::OffsetDateTime,
                new_token: $crate::ports::NewRefreshToken,
            ) -> Result<(), $crate::error::RepositoryError> {
                <$crate::adapters::persistence::unsupported::UnsupportedAuthRepository as $crate::ports::AuthRepository>::rotate_refresh_token(&self.delegate, token_id, rotated_at, new_token).await
            }

            async fn revoke_refresh_token_family(
                &self,
                family_id: $crate::domain::RefreshTokenFamilyId,
                revoked_at: time::OffsetDateTime,
            ) -> Result<usize, $crate::error::RepositoryError> {
                <$crate::adapters::persistence::unsupported::UnsupportedAuthRepository as $crate::ports::AuthRepository>::revoke_refresh_token_family(&self.delegate, family_id, revoked_at).await
            }

            async fn record_audit_event(
                &self,
                event: $crate::ports::NewAuditEvent,
            ) -> Result<$crate::domain::AuditEvent, $crate::error::RepositoryError> {
                <$crate::adapters::persistence::unsupported::UnsupportedAuthRepository as $crate::ports::AuthRepository>::record_audit_event(&self.delegate, event).await
            }
        }
    };
}

#[cfg(any(feature = "postgres", feature = "surrealdb"))]
pub(crate) use unsupported_repository_adapter;
