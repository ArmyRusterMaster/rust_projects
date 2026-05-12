use async_trait::async_trait;
use secrecy::{ExposeSecret, SecretString};
use sqlx::{Row, SqlitePool, sqlite::SqlitePoolOptions};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    domain::{
        AccessToken, AccessTokenId, AuditEvent, AuditEventKind, Email, RefreshToken,
        RefreshTokenFamilyId, RefreshTokenId, Session, SessionId, UserRecord, UserId,
    },
    error::RepositoryError,
    ports::{AuthRepository, NewAccessToken, NewAuditEvent, NewRefreshToken, NewSession, NewUser},
};

#[derive(Clone, Debug)]
pub struct SqliteAuthRepository {
    pool: SqlitePool,
}

impl SqliteAuthRepository {
    pub async fn connect(database_url: impl AsRef<str>) -> Result<Self, RepositoryError> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(database_url.as_ref())
            .await
            .map_err(map_sqlx_error)?;

        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&pool)
            .await
            .map_err(map_sqlx_error)?;

        Ok(Self { pool })
    }

    pub fn from_pool(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn initialize_schema(&self) -> Result<(), RepositoryError> {
        let statements = [
            r#"CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                email TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                disabled_at INTEGER
            )"#,
            r#"CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL,
                revoked_at INTEGER,
                user_agent TEXT,
                ip_address TEXT,
                FOREIGN KEY (user_id) REFERENCES users(id)
            )"#,
            r#"CREATE TABLE IF NOT EXISTS access_tokens (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                session_id TEXT NOT NULL,
                token_hash TEXT NOT NULL UNIQUE,
                issued_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL,
                revoked_at INTEGER,
                scopes_json TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id),
                FOREIGN KEY (session_id) REFERENCES sessions(id)
            )"#,
            r#"CREATE TABLE IF NOT EXISTS refresh_tokens (
                id TEXT PRIMARY KEY,
                family_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                session_id TEXT NOT NULL,
                token_hash TEXT NOT NULL UNIQUE,
                issued_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL,
                rotated_at INTEGER,
                revoked_at INTEGER,
                replaced_by TEXT,
                FOREIGN KEY (user_id) REFERENCES users(id),
                FOREIGN KEY (session_id) REFERENCES sessions(id)
            )"#,
            r#"CREATE TABLE IF NOT EXISTS audit_events (
                id TEXT PRIMARY KEY,
                user_id TEXT,
                kind TEXT NOT NULL,
                occurred_at INTEGER NOT NULL,
                metadata TEXT
            )"#,
            "CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)",
            "CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id)",
            "CREATE INDEX IF NOT EXISTS idx_access_tokens_hash ON access_tokens(token_hash)",
            "CREATE INDEX IF NOT EXISTS idx_refresh_tokens_hash ON refresh_tokens(token_hash)",
            "CREATE INDEX IF NOT EXISTS idx_refresh_tokens_family ON refresh_tokens(family_id)",
        ];

        for statement in statements {
            sqlx::query(statement)
                .execute(&self.pool)
                .await
                .map_err(map_sqlx_error)?;
        }

        Ok(())
    }
}

#[async_trait]
impl AuthRepository for SqliteAuthRepository {
    async fn create_user(&self, user: NewUser) -> Result<UserRecord, RepositoryError> {
        let result = sqlx::query(
            "INSERT INTO users (id, email, password_hash, created_at, disabled_at) VALUES (?, ?, ?, ?, NULL)",
        )
        .bind(user.id.to_string())
        .bind(user.email.as_str())
        .bind(user.password_hash.expose_secret())
        .bind(user.created_at.unix_timestamp())
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(UserRecord {
                id: user.id,
                email: user.email,
                password_hash: user.password_hash,
                created_at: user.created_at,
                disabled_at: None,
            }),
            Err(error) if is_unique_conflict(&error, "users.email") => {
                Err(RepositoryError::Conflict { entity: "user" })
            }
            Err(error) => Err(map_sqlx_error(error)),
        }
    }

    async fn find_user_by_email(&self, email: &Email) -> Result<Option<UserRecord>, RepositoryError> {
        let row = sqlx::query(
            "SELECT id, email, password_hash, created_at, disabled_at FROM users WHERE email = ?",
        )
        .bind(email.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.map(map_user_row).transpose()
    }

    async fn create_session(&self, session: NewSession) -> Result<Session, RepositoryError> {
        sqlx::query(
            "INSERT INTO sessions (id, user_id, created_at, expires_at, revoked_at, user_agent, ip_address)
             VALUES (?, ?, ?, ?, NULL, ?, ?)",
        )
        .bind(session.id.to_string())
        .bind(session.user_id.to_string())
        .bind(session.created_at.unix_timestamp())
        .bind(session.expires_at.unix_timestamp())
        .bind(&session.user_agent)
        .bind(&session.ip_address)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(Session {
            id: session.id,
            user_id: session.user_id,
            created_at: session.created_at,
            expires_at: session.expires_at,
            revoked_at: None,
            user_agent: session.user_agent,
            ip_address: session.ip_address,
        })
    }

    async fn find_session(
        &self,
        session_id: SessionId,
    ) -> Result<Option<Session>, RepositoryError> {
        let row = sqlx::query(
            "SELECT id, user_id, created_at, expires_at, revoked_at, user_agent, ip_address
             FROM sessions WHERE id = ?",
        )
        .bind(session_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.map(map_session_row).transpose()
    }

    async fn revoke_session(
        &self,
        session_id: SessionId,
        revoked_at: OffsetDateTime,
    ) -> Result<(), RepositoryError> {
        let result =
            sqlx::query("UPDATE sessions SET revoked_at = COALESCE(revoked_at, ?) WHERE id = ?")
                .bind(revoked_at.unix_timestamp())
                .bind(session_id.to_string())
                .execute(&self.pool)
                .await
                .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound { entity: "session" });
        }

        Ok(())
    }

    async fn create_access_token(
        &self,
        token: NewAccessToken,
    ) -> Result<AccessToken, RepositoryError> {
        let scopes_json = serde_json::to_string(&token.scopes)
            .map_err(|error| RepositoryError::Internal(error.to_string()))?;

        let result = sqlx::query(
            "INSERT INTO access_tokens
             (id, user_id, session_id, token_hash, issued_at, expires_at, revoked_at, scopes_json)
             VALUES (?, ?, ?, ?, ?, ?, NULL, ?)",
        )
        .bind(token.id.to_string())
        .bind(token.user_id.to_string())
        .bind(token.session_id.to_string())
        .bind(&token.token_hash)
        .bind(token.issued_at.unix_timestamp())
        .bind(token.expires_at.unix_timestamp())
        .bind(&scopes_json)
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(AccessToken {
                id: token.id,
                user_id: token.user_id,
                session_id: token.session_id,
                token_hash: token.token_hash,
                issued_at: token.issued_at,
                expires_at: token.expires_at,
                revoked_at: None,
                scopes: token.scopes,
            }),
            Err(error) if is_unique_conflict(&error, "access_tokens.token_hash") => {
                Err(RepositoryError::Conflict {
                    entity: "access_token",
                })
            }
            Err(error) => Err(map_sqlx_error(error)),
        }
    }

    async fn find_access_token_by_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<AccessToken>, RepositoryError> {
        let row = sqlx::query(
            "SELECT id, user_id, session_id, token_hash, issued_at, expires_at, revoked_at, scopes_json
             FROM access_tokens WHERE token_hash = ?",
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.map(map_access_token_row).transpose()
    }

    async fn create_refresh_token(
        &self,
        token: NewRefreshToken,
    ) -> Result<RefreshToken, RepositoryError> {
        let result = sqlx::query(
            "INSERT INTO refresh_tokens
             (id, family_id, user_id, session_id, token_hash, issued_at, expires_at, rotated_at, revoked_at, replaced_by)
             VALUES (?, ?, ?, ?, ?, ?, ?, NULL, NULL, NULL)",
        )
        .bind(token.id.to_string())
        .bind(token.family_id.to_string())
        .bind(token.user_id.to_string())
        .bind(token.session_id.to_string())
        .bind(&token.token_hash)
        .bind(token.issued_at.unix_timestamp())
        .bind(token.expires_at.unix_timestamp())
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(RefreshToken {
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
            }),
            Err(error) if is_unique_conflict(&error, "refresh_tokens.token_hash") => {
                Err(RepositoryError::Conflict {
                    entity: "refresh_token",
                })
            }
            Err(error) => Err(map_sqlx_error(error)),
        }
    }

    async fn find_refresh_token_by_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<RefreshToken>, RepositoryError> {
        let row = sqlx::query(
            "SELECT id, family_id, user_id, session_id, token_hash, issued_at, expires_at, rotated_at, revoked_at, replaced_by
             FROM refresh_tokens WHERE token_hash = ?",
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.map(map_refresh_token_row).transpose()
    }

    async fn rotate_refresh_token(
        &self,
        token_id: RefreshTokenId,
        rotated_at: OffsetDateTime,
        new_token: NewRefreshToken,
    ) -> Result<(), RepositoryError> {
        let mut tx = self.pool.begin().await.map_err(map_sqlx_error)?;
        let result = sqlx::query(
            "UPDATE refresh_tokens
             SET rotated_at = COALESCE(rotated_at, ?), replaced_by = COALESCE(replaced_by, ?)
             WHERE id = ?",
        )
        .bind(rotated_at.unix_timestamp())
        .bind(new_token.id.to_string())
        .bind(token_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound {
                entity: "refresh_token",
            });
        }

        sqlx::query(
            "INSERT INTO refresh_tokens
             (id, family_id, user_id, session_id, token_hash, issued_at, expires_at, rotated_at, revoked_at, replaced_by)
             VALUES (?, ?, ?, ?, ?, ?, ?, NULL, NULL, NULL)",
        )
        .bind(new_token.id.to_string())
        .bind(new_token.family_id.to_string())
        .bind(new_token.user_id.to_string())
        .bind(new_token.session_id.to_string())
        .bind(new_token.token_hash)
        .bind(new_token.issued_at.unix_timestamp())
        .bind(new_token.expires_at.unix_timestamp())
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_error)?;

        tx.commit().await.map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn revoke_refresh_token_family(
        &self,
        family_id: RefreshTokenFamilyId,
        revoked_at: OffsetDateTime,
    ) -> Result<usize, RepositoryError> {
        let result = sqlx::query(
            "UPDATE refresh_tokens
             SET revoked_at = COALESCE(revoked_at, ?)
             WHERE family_id = ? AND revoked_at IS NULL",
        )
        .bind(revoked_at.unix_timestamp())
        .bind(family_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(result.rows_affected() as usize)
    }

    async fn record_audit_event(
        &self,
        event: NewAuditEvent,
    ) -> Result<AuditEvent, RepositoryError> {
        sqlx::query(
            "INSERT INTO audit_events (id, user_id, kind, occurred_at, metadata) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(event.id.to_string())
        .bind(event.user_id.map(|value| value.to_string()))
        .bind(audit_kind_to_str(&event.kind))
        .bind(event.occurred_at.unix_timestamp())
        .bind(&event.metadata)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(AuditEvent {
            id: event.id,
            user_id: event.user_id,
            kind: event.kind,
            occurred_at: event.occurred_at,
            metadata: event.metadata,
        })
    }
}

fn map_user_row(row: sqlx::sqlite::SqliteRow) -> Result<UserRecord, RepositoryError> {
    Ok(UserRecord {
        id: parse_uuid_wrapper(row.get("id"), "user_id", UserId::from_uuid)?,
        email: Email::parse(row.get::<String, _>("email"))
            .map_err(|error| RepositoryError::Internal(error.to_string()))?,
        password_hash: SecretString::new(row.get::<String, _>("password_hash").into()),
        created_at: from_unix(row.get("created_at"))?,
        disabled_at: row
            .get::<Option<i64>, _>("disabled_at")
            .map(from_unix)
            .transpose()?,
    })
}

fn map_session_row(row: sqlx::sqlite::SqliteRow) -> Result<Session, RepositoryError> {
    Ok(Session {
        id: parse_uuid_wrapper(row.get("id"), "session_id", SessionId::from_uuid)?,
        user_id: parse_uuid_wrapper(row.get("user_id"), "user_id", UserId::from_uuid)?,
        created_at: from_unix(row.get("created_at"))?,
        expires_at: from_unix(row.get("expires_at"))?,
        revoked_at: row
            .get::<Option<i64>, _>("revoked_at")
            .map(from_unix)
            .transpose()?,
        user_agent: row.get("user_agent"),
        ip_address: row.get("ip_address"),
    })
}

fn map_access_token_row(row: sqlx::sqlite::SqliteRow) -> Result<AccessToken, RepositoryError> {
    let scopes_json = row.get::<String, _>("scopes_json");
    let scopes = serde_json::from_str::<Vec<String>>(&scopes_json)
        .map_err(|error| RepositoryError::Internal(error.to_string()))?;

    Ok(AccessToken {
        id: parse_uuid_wrapper(row.get("id"), "access_token_id", AccessTokenId::from_uuid)?,
        user_id: parse_uuid_wrapper(row.get("user_id"), "user_id", UserId::from_uuid)?,
        session_id: parse_uuid_wrapper(row.get("session_id"), "session_id", SessionId::from_uuid)?,
        token_hash: row.get("token_hash"),
        issued_at: from_unix(row.get("issued_at"))?,
        expires_at: from_unix(row.get("expires_at"))?,
        revoked_at: row
            .get::<Option<i64>, _>("revoked_at")
            .map(from_unix)
            .transpose()?,
        scopes,
    })
}

fn map_refresh_token_row(row: sqlx::sqlite::SqliteRow) -> Result<RefreshToken, RepositoryError> {
    Ok(RefreshToken {
        id: parse_uuid_wrapper(row.get("id"), "refresh_token_id", RefreshTokenId::from_uuid)?,
        family_id: parse_uuid_wrapper(
            row.get("family_id"),
            "refresh_token_family_id",
            RefreshTokenFamilyId::from_uuid,
        )?,
        user_id: parse_uuid_wrapper(row.get("user_id"), "user_id", UserId::from_uuid)?,
        session_id: parse_uuid_wrapper(row.get("session_id"), "session_id", SessionId::from_uuid)?,
        token_hash: row.get("token_hash"),
        issued_at: from_unix(row.get("issued_at"))?,
        expires_at: from_unix(row.get("expires_at"))?,
        rotated_at: row
            .get::<Option<i64>, _>("rotated_at")
            .map(from_unix)
            .transpose()?,
        revoked_at: row
            .get::<Option<i64>, _>("revoked_at")
            .map(from_unix)
            .transpose()?,
        replaced_by: row
            .get::<Option<String>, _>("replaced_by")
            .map(|value| parse_uuid_wrapper(value, "replaced_by", RefreshTokenId::from_uuid))
            .transpose()?,
    })
}

fn from_unix(value: i64) -> Result<OffsetDateTime, RepositoryError> {
    OffsetDateTime::from_unix_timestamp(value)
        .map_err(|error| RepositoryError::Internal(error.to_string()))
}

fn parse_uuid_wrapper<T>(
    raw: String,
    field: &'static str,
    ctor: fn(Uuid) -> T,
) -> Result<T, RepositoryError> {
    let parsed = Uuid::parse_str(&raw)
        .map_err(|error| RepositoryError::Internal(format!("{field}: {error}")))?;
    Ok(ctor(parsed))
}

fn map_sqlx_error(error: sqlx::Error) -> RepositoryError {
    RepositoryError::Internal(error.to_string())
}

fn is_unique_conflict(error: &sqlx::Error, fragment: &str) -> bool {
    match error {
        sqlx::Error::Database(db_error) => {
            let message = db_error.message();
            message.contains("UNIQUE constraint failed") && message.contains(fragment)
        }
        _ => false,
    }
}

fn audit_kind_to_str(kind: &AuditEventKind) -> &'static str {
    match kind {
        AuditEventKind::UserRegistered => "user_registered",
        AuditEventKind::LoginSucceeded => "login_succeeded",
        AuditEventKind::LoginFailed => "login_failed",
        AuditEventKind::TokenRefreshed => "token_refreshed",
        AuditEventKind::RefreshTokenReuseDetected => "refresh_token_reuse_detected",
        AuditEventKind::Logout => "logout",
    }
}
