# Security Best Practices

## Пароли

- Хешировать пароли через Argon2id.
- Использовать уникальную соль для каждого пароля.
- Настраивать параметры Argon2 под целевое железо.
- Не логировать пароли, токены, authorization codes и secrets.
- Проверять пароли по спискам известных утечек, если продукт это допускает.

## MFA

- Поддерживать TOTP, WebAuthn/passkeys или оба варианта.
- Recovery codes хранить только в хешированном виде.
- MFA reset делать через усиленную проверку.
- Для рискованных операций использовать step-up authentication.

## Rate limiting

Ограничивать:

- Login attempts по user id, email, IP и device fingerprint.
- Password reset.
- MFA verification.
- Token refresh.
- Device code polling.

## Audit log

Логировать события:

- login success/failure
- logout
- refresh token rotation
- token reuse detection
- password change
- MFA changes
- passkey registration/removal
- role or permission changes
- admin impersonation

Audit log должен быть append-only на уровне приложения.

## Secrets

- Secrets хранить в secret manager или переменных окружения.
- Не коммитить `.env` с реальными значениями.
- Ротировать signing keys и client secrets.
- Разделять secrets для dev, staging и production.

## Multi-tenancy

- `tenant_id` должен быть частью модели данных и авторизационных проверок.
- Нельзя доверять `tenant_id` только из запроса клиента.
- Все уникальные индексы и запросы должны учитывать tenant boundary, если данные tenant-scoped.

## Ошибки API

Ошибки login не должны раскрывать, существует ли пользователь.

Пример:

```json
{
  "error": "invalid_credentials"
}
```

## Production defaults

- TLS везде.
- HSTS для браузерных клиентов.
- Secure cookies.
- Structured logging.
- Metrics по latency, error rate, auth failures и token refresh.
- Alerts на brute force, token reuse и аномальные admin actions.
