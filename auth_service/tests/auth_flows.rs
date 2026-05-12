use auth_service::{
    AuthConfig, AuthContext, AuthError, AuthService,
    adapters::{
        crypto::{Argon2CredentialHasher, RandomTokenGenerator},
        persistence::InMemoryAuthRepository,
    },
    application::{LoginRequest, LogoutRequest, RefreshRequest, RegisterRequest},
    ports::{Clock, CredentialHasher},
};
use secrecy::{ExposeSecret, SecretString};
use time::OffsetDateTime;

const PASSWORD: &str = "correct horse battery staple";

type TestService =
    AuthService<InMemoryAuthRepository, TestCredentialHasher, RandomTokenGenerator, FixedClock>;

#[derive(Clone, Debug)]
struct TestCredentialHasher;

impl CredentialHasher for TestCredentialHasher {
    fn hash_password(&self, password: &str) -> Result<SecretString, AuthError> {
        Ok(SecretString::new(format!("test-hash:{password}").into()))
    }

    fn verify_password(&self, password: &str, password_hash: &SecretString) -> Result<bool, AuthError> {
        Ok(password_hash.expose_secret() == format!("test-hash:{password}"))
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

fn test_service() -> (TestService, InMemoryAuthRepository) {
    let repository = InMemoryAuthRepository::new();
    let service = AuthService::new(
        repository.clone(),
        TestCredentialHasher,
        RandomTokenGenerator::default(),
        FixedClock {
            now: OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap(),
        },
        AuthConfig::default(),
    );

    (service, repository)
}

async fn register_user(service: &TestService) {
    service
        .register(RegisterRequest {
            email: "alice@example.com".to_owned(),
            password: PASSWORD.to_owned(),
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn register_creates_user_and_audit_event() {
    let (service, repository) = test_service();

    let user = service
        .register(RegisterRequest {
            email: "Alice@Example.com".to_owned(),
            password: PASSWORD.to_owned(),
        })
        .await
        .unwrap();

    assert_eq!(user.email.as_str(), "alice@example.com");
    assert_eq!(repository.audit_event_count().unwrap(), 1);
}

#[tokio::test]
async fn register_rejects_weak_password() {
    let (service, _) = test_service();

    let result = service
        .register(RegisterRequest {
            email: "alice@example.com".to_owned(),
            password: "short".to_owned(),
        })
        .await;

    assert!(matches!(result, Err(AuthError::WeakPassword)));
}

#[tokio::test]
async fn register_rejects_duplicate_email() {
    let (service, _) = test_service();
    register_user(&service).await;

    let result = service
        .register(RegisterRequest {
            email: "ALICE@example.com".to_owned(),
            password: PASSWORD.to_owned(),
        })
        .await;

    assert!(matches!(result, Err(AuthError::UserAlreadyExists)));
}

#[tokio::test]
async fn login_returns_tokens_and_introspection_is_active() {
    let (service, _) = test_service();
    register_user(&service).await;

    let authenticated = service
        .login(
            LoginRequest {
                email: "alice@example.com".to_owned(),
                password: PASSWORD.to_owned(),
            },
            AuthContext {
                user_agent: Some("tests".to_owned()),
                ip_address: Some("127.0.0.1".to_owned()),
            },
        )
        .await
        .unwrap();

    let introspection = service
        .introspect_access_token(&authenticated.tokens.access_token)
        .await
        .unwrap();

    assert_eq!(authenticated.tokens.token_type, "Bearer");
    assert!(introspection.active);
    assert_eq!(introspection.user_id, Some(authenticated.user.id));
}

#[tokio::test]
async fn login_rejects_wrong_password() {
    let (service, _) = test_service();
    register_user(&service).await;

    let result = service
        .login(
            LoginRequest {
                email: "alice@example.com".to_owned(),
                password: "wrong password value".to_owned(),
            },
            AuthContext::default(),
        )
        .await;

    assert!(matches!(result, Err(AuthError::InvalidCredentials)));
}

#[tokio::test]
async fn refresh_rotates_token_and_reuse_revokes_family() {
    let (service, _) = test_service();
    register_user(&service).await;
    let authenticated = service
        .login(
            LoginRequest {
                email: "alice@example.com".to_owned(),
                password: PASSWORD.to_owned(),
            },
            AuthContext::default(),
        )
        .await
        .unwrap();

    let old_refresh_token = authenticated.tokens.refresh_token;
    let refreshed = service
        .refresh(RefreshRequest {
            refresh_token: old_refresh_token.clone(),
        })
        .await
        .unwrap();

    assert_ne!(refreshed.refresh_token, old_refresh_token);

    let reuse_result = service
        .refresh(RefreshRequest {
            refresh_token: old_refresh_token,
        })
        .await;
    assert!(matches!(reuse_result, Err(AuthError::TokenReuseDetected)));

    let current_result = service
        .refresh(RefreshRequest {
            refresh_token: refreshed.refresh_token,
        })
        .await;
    assert!(matches!(current_result, Err(AuthError::TokenRevoked)));
}

#[tokio::test]
async fn logout_revokes_session_for_access_token_introspection() {
    let (service, _) = test_service();
    register_user(&service).await;
    let authenticated = service
        .login(
            LoginRequest {
                email: "alice@example.com".to_owned(),
                password: PASSWORD.to_owned(),
            },
            AuthContext::default(),
        )
        .await
        .unwrap();

    let before_logout = service
        .introspect_access_token(&authenticated.tokens.access_token)
        .await
        .unwrap();
    assert!(before_logout.active);

    service
        .logout(LogoutRequest {
            refresh_token: authenticated.tokens.refresh_token.clone(),
        })
        .await
        .unwrap();

    let after_logout = service
        .introspect_access_token(&authenticated.tokens.access_token)
        .await
        .unwrap();
    assert!(!after_logout.active);

    let refresh_result = service
        .refresh(RefreshRequest {
            refresh_token: authenticated.tokens.refresh_token,
        })
        .await;
    assert!(matches!(refresh_result, Err(AuthError::TokenRevoked)));
}

#[test]
fn argon2_hasher_verifies_passwords() {
    let hasher = Argon2CredentialHasher;
    let hash = hasher.hash_password(PASSWORD).unwrap();

    assert!(hasher.verify_password(PASSWORD, &hash).unwrap());
    assert!(
        !hasher
            .verify_password("wrong password value", &hash)
            .unwrap()
    );
}
