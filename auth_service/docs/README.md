# Auth Service Docs

Эта папка содержит рабочие заметки для проектирования и реализации сервиса аутентификации и авторизации на Rust.

## Файлы

- `oauth2.md` - роли, grant types и ошибки внедрения OAuth 2.0.
- `openid-connect.md` - как добавлять идентификацию поверх OAuth 2.0.
- `jwt-and-tokens.md` - access tokens, refresh tokens, JWT, JWK и ротация ключей.
- `sessions-and-cookies.md` - серверные сессии, cookie и защита браузерных клиентов.
- `saml2.md` - когда нужен SAML 2.0 и как его безопасно подключать.
- `webauthn-passkeys.md` - passkeys, FIDO2 и passwordless-вход.
- `authorization-models.md` - RBAC, ABAC, ReBAC и policy-based authorization.
- `best-practices.md` - практики безопасности для продакшена.
- `rust-libraries.md` - полезные библиотеки Rust для реализации.
- `implementation-plan.md` - план реализации расширяемой архитектуры и тестирования.
- `implementation-checklist.md` - практический чеклист перед запуском.
- `current-state.md` - текущее состояние проекта, ограничения и следующие шаги.
- `static-linking.md` - настройка и проверка статической линковки (`musl`).

## Базовая архитектура

Минимальный auth service обычно включает:

- HTTP API для login, logout, refresh, introspection, userinfo и управления ключами.
- Хранилище пользователей, хешей паролей, сессий, refresh-токенов и audit log.
- Подсистему выпуска и проверки токенов.
- Политику прав доступа, отделенную от бизнес-логики.
- Ротацию signing keys и endpoint JWKS, если используются JWT.
- Метрики, структурные логи и алерты на подозрительные события.

## Главный принцип

Аутентификация отвечает на вопрос "кто это", авторизация отвечает на вопрос "что этому субъекту можно делать". В коде и API эти границы лучше держать явно.
