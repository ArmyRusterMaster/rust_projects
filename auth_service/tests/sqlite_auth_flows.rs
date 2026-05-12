#![cfg(feature = "sqlite")]

use auth_service::{
    AuthConfig, AuthContext, AuthError, AuthService,
    adapters::{crypto::RandomTokenGenerator, persistence::SqliteAuthRepository},
    application::{LoginRequest, LogoutRequest, RefreshRequest, RegisterRequest},
    ports::{Clock, CredentialHasher},
};
use time::OffsetDateTime;

const PASSWORD: &str = "correct horse battery staple";

#[derive(Clone, Debug)]
struct TestCredentialHasher;

impl CredentialHasher for TestCredentialHasher {
    fn hash_password(&self, password: &str) -> Result<String, AuthError> {
        Ok(format!("test-hash:{password}"))
    }

    fn verify_password(&self, password: &str, password_hash: &str) -> Result<bool, AuthError> {
        Ok(password_hash == format!("test-hash:{password}"))
    }
}

#[derive(Clone, Copy, Debug)]
struct FixedClock {
    now: OffsetDateTime,
}

impl Clock for FixedClock {
    fn now(&self) -> OffsetDateTime {
        self.now
    }
}

#[tokio::test]
async fn sqlite_auth_flow_works_end_to_end() {
    let repository = SqliteAuthRepository::connect("sqlite::memory:")
        .await
        .unwrap();
    repository.initialize_schema().await.unwrap();

    let service = AuthService::new(
        repository,
        TestCredentialHasher,
        RandomTokenGenerator::default(),
        FixedClock {
            now: OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap(),
        },
        AuthConfig::default(),
    );

    service
        .register(RegisterRequest {
            email: "alice@example.com".to_owned(),
            password: PASSWORD.to_owned(),
        })
        .await
        .unwrap();

    let authenticated = service
        .login(
            LoginRequest {
                email: "alice@example.com".to_owned(),
                password: PASSWORD.to_owned(),
            },
            AuthContext {
                user_agent: Some("sqlite-test".to_owned()),
                ip_address: Some("127.0.0.1".to_owned()),
            },
        )
        .await
        .unwrap();

    let before_logout = service
        .introspect_access_token(&authenticated.tokens.access_token)
        .await
        .unwrap();
    assert!(before_logout.active);

    let old_refresh_token = authenticated.tokens.refresh_token.clone();
    let refreshed = service
        .refresh(RefreshRequest {
            refresh_token: old_refresh_token.clone(),
        })
        .await
        .unwrap();
    assert_ne!(old_refresh_token, refreshed.refresh_token);

    service
        .logout(LogoutRequest {
            refresh_token: refreshed.refresh_token.clone(),
        })
        .await
        .unwrap();

    let after_logout = service
        .introspect_access_token(&authenticated.tokens.access_token)
        .await
        .unwrap();
    assert!(!after_logout.active);
}
