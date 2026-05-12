# Security And Architecture Summary

## Authorization model
- Базовая стратегия: deny-by-default.
- Выбор модели:
  - RBAC для простых стабильных ролей,
  - ABAC для контекстных правил,
  - ReBAC для graph/collaboration сценариев.
- Политики должны быть отделены от HTTP handlers и покрыты unit-тестами.

## Security baseline
- Пароли: Argon2id.
- Секреты: `secrecy::SecretString`, не логировать sensitive data.
- Audit log: login/logout/refresh/reuse/security changes.
- Rate limiting: login/refresh/reset/MFA.
- Multi-tenant boundary проверять в каждой операции.

## Реализовано в проекте
- Ports/adapters архитектура.
- In-memory + SQLite repository.
- Use-cases: register/login/refresh/logout/introspect.
- Refresh rotation с atomic contract.
- HTTP API (axum) с централизованным error mapping.
- SQLite schema + integration contract tests.

## Следующие шаги
1. Production adapters (`postgres`, `surrealdb`).
2. Rate limiting, MFA, password reset.
3. Policy-based authorization слой.
4. OIDC/JWKS endpoints.
