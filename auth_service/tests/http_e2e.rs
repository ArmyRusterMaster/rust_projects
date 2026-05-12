use std::sync::Arc;

use auth_service::{
    AuthConfig, AuthService,
    adapters::{
        crypto::{Argon2CredentialHasher, RandomTokenGenerator},
        persistence::InMemoryAuthRepository,
    },
    domain::TokenIntrospection,
    http,
    ports::SystemClock,
    server::DynAuthService,
};
use reqwest::StatusCode;
use serde_json::json;

async fn spawn_app() -> (String, tokio::task::JoinHandle<()>) {
    let service = AuthService::new(
        InMemoryAuthRepository::new(),
        Argon2CredentialHasher,
        RandomTokenGenerator::default(),
        SystemClock,
        AuthConfig::default(),
    );
    let service: Arc<DynAuthService> = Arc::new(service);
    let app = http::build_router(service, 10_000);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (format!("http://{}", addr), handle)
}

#[tokio::test]
async fn e2e_auth_happy_path() {
    let (base_url, _server) = spawn_app().await;
    let client = reqwest::Client::new();

    let register = client
        .post(format!("{base_url}/register"))
        .json(&json!({
            "email": "alice@example.com",
            "password": "correct horse battery staple"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(register.status(), StatusCode::CREATED);

    let login = client
        .post(format!("{base_url}/login"))
        .json(&json!({
            "email": "alice@example.com",
            "password": "correct horse battery staple"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(login.status(), StatusCode::OK);
    let login_body: serde_json::Value = login.json().await.unwrap();

    let access_token = login_body["tokens"]["access_token"].as_str().unwrap().to_owned();
    let refresh_token = login_body["tokens"]["refresh_token"].as_str().unwrap().to_owned();

    let introspect_before = client
        .post(format!("{base_url}/introspect"))
        .json(&json!({ "access_token": access_token }))
        .send()
        .await
        .unwrap();
    assert_eq!(introspect_before.status(), StatusCode::OK);
    let introspection: TokenIntrospection = introspect_before.json().await.unwrap();
    assert!(introspection.active);

    let refresh = client
        .post(format!("{base_url}/refresh"))
        .json(&json!({ "refresh_token": refresh_token }))
        .send()
        .await
        .unwrap();
    assert_eq!(refresh.status(), StatusCode::OK);
    let refreshed: serde_json::Value = refresh.json().await.unwrap();

    let logout = client
        .post(format!("{base_url}/logout"))
        .json(&json!({ "refresh_token": refreshed["refresh_token"].as_str().unwrap() }))
        .send()
        .await
        .unwrap();
    assert_eq!(logout.status(), StatusCode::NO_CONTENT);

    let introspect_after = client
        .post(format!("{base_url}/introspect"))
        .json(&json!({ "access_token": login_body["tokens"]["access_token"] }))
        .send()
        .await
        .unwrap();
    assert_eq!(introspect_after.status(), StatusCode::OK);
    let introspection_after: TokenIntrospection = introspect_after.json().await.unwrap();
    assert!(!introspection_after.active);
}

#[tokio::test]
async fn e2e_rejects_invalid_login() {
    let (base_url, _server) = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{base_url}/login"))
        .json(&json!({
            "email": "nobody@example.com",
            "password": "wrong password"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
