use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use async_trait::async_trait;
use time::OffsetDateTime;

use crate::{
    Email,
    domain::{
        AccessToken, AccessTokenId, AuditEvent, AuditEventId, RefreshToken, RefreshTokenFamilyId,
        RefreshTokenId, Session, SessionId, UserRecord, UserId,
    },
    error::RepositoryError,
    ports::{AuthRepository, NewAccessToken, NewAuditEvent, NewRefreshToken, NewSession, NewUser},
};

#[derive(Clone, Debug, Default)]
pub struct InMemoryAuthRepository {
    state: Arc<RwLock<InMemoryState>>,
}

#[derive(Clone, Debug, Default)]
struct InMemoryState {
    users: HashMap<UserId, UserRecord>,
    users_by_email: HashMap<String, UserId>,
    sessions: HashMap<SessionId, Session>,
    access_tokens: HashMap<AccessTokenId, AccessToken>,
    access_tokens_by_hash: HashMap<String, AccessTokenId>,
    refresh_tokens: HashMap<RefreshTokenId, RefreshToken>,
    refresh_tokens_by_hash: HashMap<String, RefreshTokenId>,
    audit_events: HashMap<AuditEventId, AuditEvent>,
}

impl InMemoryAuthRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn audit_event_count(&self) -> Result<usize, RepositoryError> {
        Ok(self.read_state()?.audit_events.len())
    }

    fn read_state(&self) -> Result<RwLockReadGuard<'_, InMemoryState>, RepositoryError> {
        self.state
            .read()
            .map_err(|error| RepositoryError::Internal(error.to_string()))
    }

    fn write_state(&self) -> Result<RwLockWriteGuard<'_, InMemoryState>, RepositoryError> {
        self.state
            .write()
            .map_err(|error| RepositoryError::Internal(error.to_string()))
    }
}

#[async_trait]
impl AuthRepository for InMemoryAuthRepository {
    async fn create_user(&self, user: NewUser) -> Result<UserRecord, RepositoryError> {
        let mut state = self.write_state()?;
        let email_key = user.email.as_str().to_owned();

        if state.users_by_email.contains_key(&email_key) {
            return Err(RepositoryError::Conflict { entity: "user" });
        }

        let user = UserRecord {
            id: user.id,
            email: user.email,
            password_hash: user.password_hash,
            created_at: user.created_at,
            disabled_at: None,
        };

        state.users_by_email.insert(email_key, user.id);
        state.users.insert(user.id, user.clone());

        Ok(user)
    }

    async fn find_user_by_email(&self, email: &Email) -> Result<Option<UserRecord>, RepositoryError> {
        let state = self.read_state()?;
        let Some(user_id) = state.users_by_email.get(email.as_str()) else {
            return Ok(None);
        };

        Ok(state.users.get(user_id).cloned())
    }

    async fn create_session(&self, session: NewSession) -> Result<Session, RepositoryError> {
        let mut state = self.write_state()?;

        if state.sessions.contains_key(&session.id) {
            return Err(RepositoryError::Conflict { entity: "session" });
        }

        let session = Session {
            id: session.id,
            user_id: session.user_id,
            created_at: session.created_at,
            expires_at: session.expires_at,
            revoked_at: None,
            user_agent: session.user_agent,
            ip_address: session.ip_address,
        };

        state.sessions.insert(session.id, session.clone());

        Ok(session)
    }

    async fn find_session(
        &self,
        session_id: SessionId,
    ) -> Result<Option<Session>, RepositoryError> {
        Ok(self.read_state()?.sessions.get(&session_id).cloned())
    }

    async fn revoke_session(
        &self,
        session_id: SessionId,
        revoked_at: OffsetDateTime,
    ) -> Result<(), RepositoryError> {
        let mut state = self.write_state()?;
        let Some(session) = state.sessions.get_mut(&session_id) else {
            return Err(RepositoryError::NotFound { entity: "session" });
        };

        session.revoked_at.get_or_insert(revoked_at);

        Ok(())
    }

    async fn create_access_token(
        &self,
        token: NewAccessToken,
    ) -> Result<AccessToken, RepositoryError> {
        let mut state = self.write_state()?;

        if state.access_tokens.contains_key(&token.id)
            || state.access_tokens_by_hash.contains_key(&token.token_hash)
        {
            return Err(RepositoryError::Conflict {
                entity: "access_token",
            });
        }

        let token = AccessToken {
            id: token.id,
            user_id: token.user_id,
            session_id: token.session_id,
            token_hash: token.token_hash,
            issued_at: token.issued_at,
            expires_at: token.expires_at,
            revoked_at: None,
            scopes: token.scopes,
        };

        state
            .access_tokens_by_hash
            .insert(token.token_hash.clone(), token.id);
        state.access_tokens.insert(token.id, token.clone());

        Ok(token)
    }

    async fn find_access_token_by_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<AccessToken>, RepositoryError> {
        let state = self.read_state()?;
        let Some(token_id) = state.access_tokens_by_hash.get(token_hash) else {
            return Ok(None);
        };

        Ok(state.access_tokens.get(token_id).cloned())
    }

    async fn create_refresh_token(
        &self,
        token: NewRefreshToken,
    ) -> Result<RefreshToken, RepositoryError> {
        let mut state = self.write_state()?;

        if state.refresh_tokens.contains_key(&token.id)
            || state.refresh_tokens_by_hash.contains_key(&token.token_hash)
        {
            return Err(RepositoryError::Conflict {
                entity: "refresh_token",
            });
        }

        let token = RefreshToken {
            id: token.id,
            family_id: token.family_id,
            user_id: token.user_id,
            session_id: token.session_id,
            token_hash: token.token_hash,
            issued_at: token.issued_at,
            expires_at: token.expires_at,
            rotated_at: None,
            revoked_at: None,
            replaced_by: None,
        };

        state
            .refresh_tokens_by_hash
            .insert(token.token_hash.clone(), token.id);
        state.refresh_tokens.insert(token.id, token.clone());

        Ok(token)
    }

    async fn find_refresh_token_by_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<RefreshToken>, RepositoryError> {
        let state = self.read_state()?;
        let Some(token_id) = state.refresh_tokens_by_hash.get(token_hash) else {
            return Ok(None);
        };

        Ok(state.refresh_tokens.get(token_id).cloned())
    }

    async fn rotate_refresh_token(
        &self,
        token_id: RefreshTokenId,
        rotated_at: OffsetDateTime,
        new_token: NewRefreshToken,
    ) -> Result<(), RepositoryError> {
        let mut state = self.write_state()?;
        let Some(token) = state.refresh_tokens.get_mut(&token_id) else {
            return Err(RepositoryError::NotFound {
                entity: "refresh_token",
            });
        };

        token.rotated_at.get_or_insert(rotated_at);
        token.replaced_by.get_or_insert(new_token.id);

        if state.refresh_tokens.contains_key(&new_token.id)
            || state.refresh_tokens_by_hash.contains_key(&new_token.token_hash)
        {
            return Err(RepositoryError::Conflict {
                entity: "refresh_token",
            });
        }

        let replacement = RefreshToken {
            id: new_token.id,
            family_id: new_token.family_id,
            user_id: new_token.user_id,
            session_id: new_token.session_id,
            token_hash: new_token.token_hash,
            issued_at: new_token.issued_at,
            expires_at: new_token.expires_at,
            rotated_at: None,
            revoked_at: None,
            replaced_by: None,
        };
        state
            .refresh_tokens_by_hash
            .insert(replacement.token_hash.clone(), replacement.id);
        state.refresh_tokens.insert(replacement.id, replacement);

        Ok(())
    }

    async fn revoke_refresh_token_family(
        &self,
        family_id: RefreshTokenFamilyId,
        revoked_at: OffsetDateTime,
    ) -> Result<usize, RepositoryError> {
        let mut state = self.write_state()?;
        let mut revoked = 0;

        for token in state.refresh_tokens.values_mut() {
            if token.family_id == family_id && token.revoked_at.is_none() {
                token.revoked_at = Some(revoked_at);
                revoked += 1;
            }
        }

        Ok(revoked)
    }

    async fn record_audit_event(
        &self,
        event: NewAuditEvent,
    ) -> Result<AuditEvent, RepositoryError> {
        let mut state = self.write_state()?;

        if state.audit_events.contains_key(&event.id) {
            return Err(RepositoryError::Conflict {
                entity: "audit_event",
            });
        }

        let event = AuditEvent {
            id: event.id,
            user_id: event.user_id,
            kind: event.kind,
            occurred_at: event.occurred_at,
            metadata: event.metadata,
        };

        state.audit_events.insert(event.id, event.clone());

        Ok(event)
    }
}
