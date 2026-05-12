#![cfg(feature = "sqlite")]

use auth_service::{
    adapters::persistence::SqliteAuthRepository,
    domain::{Email, RefreshTokenFamilyId, RefreshTokenId, SessionId, UserId},
    ports::{AuthRepository, NewRefreshToken, NewSession, NewUser},
};
use secrecy::SecretString;
use time::{Duration, OffsetDateTime};

#[tokio::test]
async fn sqlite_rotate_refresh_token_is_atomic_contract() {
    let repository = SqliteAuthRepository::connect("sqlite::memory:").await.unwrap();
    repository.initialize_schema().await.unwrap();

    let now = OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap();
    let user_id = UserId::new();
    let session_id = SessionId::new();
    let family_id = RefreshTokenFamilyId::new();

    repository
        .create_user(NewUser {
            id: user_id,
            email: Email::parse("contract@example.com").unwrap(),
            password_hash: SecretString::new("hash-1".to_owned().into()),
            created_at: now,
        })
        .await
        .unwrap();

    repository
        .create_session(NewSession {
            id: session_id,
            user_id,
            created_at: now,
            expires_at: now + Duration::days(7),
            user_agent: None,
            ip_address: None,
        })
        .await
        .unwrap();

    let current = NewRefreshToken {
        id: RefreshTokenId::new(),
        family_id,
        user_id,
        session_id,
        token_hash: "current-hash".to_owned(),
        issued_at: now,
        expires_at: now + Duration::days(30),
    };

    repository.create_refresh_token(current.clone()).await.unwrap();

    let replacement = NewRefreshToken {
        id: RefreshTokenId::new(),
        family_id,
        user_id,
        session_id,
        token_hash: "replacement-hash".to_owned(),
        issued_at: now,
        expires_at: now + Duration::days(30),
    };

    repository
        .rotate_refresh_token(current.id, now, replacement.clone())
        .await
        .unwrap();

    let old = repository
        .find_refresh_token_by_hash("current-hash")
        .await
        .unwrap()
        .unwrap();
    assert!(old.rotated_at.is_some());
    assert_eq!(old.replaced_by, Some(replacement.id));

    let new = repository
        .find_refresh_token_by_hash("replacement-hash")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(new.id, replacement.id);
    assert!(new.rotated_at.is_none());
}
