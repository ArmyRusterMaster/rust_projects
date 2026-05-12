# SAML 2.0

SAML 2.0 часто встречается в enterprise SSO. Для новых consumer-facing интеграций обычно удобнее OIDC, но SAML остается важным для корпоративных клиентов.

## Основные роли

- Identity Provider - аутентифицирует пользователя.
- Service Provider - приложение, в которое пользователь входит.
- Assertion - подписанное утверждение о пользователе.

## Когда использовать

SAML полезен, если:

- Клиентская организация уже использует SAML IdP.
- Нужна интеграция с enterprise SSO.
- Требуется совместимость с существующими корпоративными политиками.

## Проверки безопасности

Обязательно проверять:

- Подпись assertion или response.
- Issuer.
- Audience.
- Recipient.
- Destination.
- NotBefore и NotOnOrAfter.
- InResponseTo для SP-initiated login.
- Уникальность assertion id для защиты от replay.

## Частые ошибки

- Принимать unsigned assertion.
- Проверять подпись XML небезопасным парсером.
- Не валидировать audience и recipient.
- Не защищаться от XML Signature Wrapping.
- Хранить metadata IdP без процесса ротации сертификатов.

## Практический совет

SAML лучше изолировать в отдельный модуль или сервисный адаптер. XML security сложна, поэтому стоит использовать зрелые библиотеки и отдельные integration tests с реальными metadata.
