# Implementation Checklist

## Domain model

- User
- Identity
- Credential
- Session
- Client
- AuthorizationCode
- AccessToken
- RefreshToken
- RefreshTokenFamily
- Scope
- Role
- Permission
- AuditEvent
- SigningKey

## API endpoints

Минимальный набор:

- `POST /login`
- `POST /logout`
- `POST /token`
- `POST /token/introspect`
- `POST /token/revoke`
- `GET /.well-known/openid-configuration`
- `GET /.well-known/jwks.json`
- `GET /userinfo`
- `POST /password/reset/request`
- `POST /password/reset/confirm`
- `POST /mfa/totp/enable`
- `POST /mfa/totp/verify`
- `POST /webauthn/register/start`
- `POST /webauthn/register/finish`
- `POST /webauthn/login/start`
- `POST /webauthn/login/finish`

## Database tables

Обычно нужны:

- users
- user_identities
- password_credentials
- sessions
- oauth_clients
- authorization_codes
- refresh_tokens
- token_families
- signing_keys
- roles
- permissions
- role_permissions
- user_roles
- audit_events
- webauthn_credentials
- mfa_factors

## Security tests

- Нельзя использовать authorization code дважды.
- Redirect URI проверяется строго.
- PKCE verifier обязателен и проверяется.
- Refresh token rotation отзывает старый токен.
- Reuse старого refresh token отзывает token family.
- JWT с неправильным issuer отклоняется.
- JWT с неправильным audience отклоняется.
- JWT с истекшим `exp` отклоняется.
- Токен с неизвестным `kid` отклоняется.
- Logout удаляет сессию и отзывает refresh tokens.
- Tenant boundary нельзя обойти через подмену id.

## Operational checklist

- Настроены TLS и secure headers.
- Есть rate limits.
- Есть audit log.
- Есть metrics и tracing.
- Есть backup/restore для базы.
- Есть key rotation playbook.
- Есть incident playbook для утечки signing key или refresh tokens.
- Есть миграции базы.
- Есть seed/dev fixtures без реальных secrets.
