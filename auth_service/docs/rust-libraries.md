# Rust Libraries

Ниже перечислены библиотеки, которые чаще всего полезны для реализации auth service на Rust. Перед фиксацией зависимостей проверьте актуальность версий, maintenance status и security advisories.

## HTTP API

- `axum` - современный web framework поверх `tower` и `hyper`.
- `tower` - middleware, rate limiting, timeout, tracing layers.
- `tower-http` - CORS, compression, request id, trace layer.
- `utoipa` - OpenAPI генерация.

## Async runtime

- `tokio` - стандартный async runtime для серверных Rust-приложений.

## Serialization and validation

- `serde` - сериализация и десериализация.
- `serde_json` - JSON.
- `validator` или `garde` - validation DTO.
- `url` - безопасная работа с URL и redirect URI.

## Database and storage

- `sqlx` - async SQL с compile-time проверками запросов.
- `diesel` - зрелый ORM/query builder.
- `deadpool` - pooling для разных backend.
- `redis` - Redis client для сессий, rate limit и кеша.

## Password hashing

- `argon2` - Argon2id для паролей.
- `password-hash` - общий формат и traits для password hashing.
- `rand` - генерация соли и случайных токенов.

## Tokens and crypto

- `jsonwebtoken` - JWT encode/decode.
- `josekit` - JOSE/JWT/JWK/JWS/JWE инструменты.
- `jwt-simple` - удобный API для JWT.
- `ed25519-dalek` - Ed25519 подписи.
- `p256` - ECDSA P-256.
- `ring` - криптографические primitives.

## OAuth 2.0 and OIDC

- `oauth2` - OAuth 2.0 client flows.
- `openidconnect` - OpenID Connect client, discovery и token validation.

Для полноценного authorization server может потребоваться своя доменная логика: регистрация клиентов, consent, scopes, grants, refresh token rotation, JWKS и token introspection.

## WebAuthn

- `webauthn-rs` - WebAuthn/FIDO2/passkeys.

## Authorization policies

- `oso` - policy engine.
- `cedar-policy` - Cedar authorization policies.
- `regorus` - Rego interpreter для Open Policy Agent policies.

## Observability

- `tracing` - structured logs и spans.
- `tracing-subscriber` - subscriber и форматирование.
- `opentelemetry` - distributed tracing.
- `metrics` - metrics facade.
- `metrics-exporter-prometheus` - Prometheus exporter.

## Security hardening

- `secrecy` - защита secrets от случайного Debug/Display.
- `zeroize` - очистка чувствительных данных из памяти.
- `constant_time_eq` - constant-time сравнение.
- `uuid` - идентификаторы сессий, клиентов и пользователей.
- `time` - работа со временем.

## Testing

- `proptest` - property-based tests.
- `wiremock` - mock HTTP-сервер.
- `testcontainers` - интеграционные тесты с PostgreSQL, Redis и другими сервисами.
- `insta` - snapshot tests для стабильных API ответов.
