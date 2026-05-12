# OpenID Connect

OpenID Connect - это слой идентификации поверх OAuth 2.0. Он добавляет ID Token, UserInfo endpoint, discovery metadata и стандартный способ получить информацию о пользователе.

## Когда нужен OIDC

OIDC нужен, если клиенту надо выполнить вход пользователя и получить подтвержденную идентичность.

Подходящие сценарии:

- Login через внешний identity provider.
- Single Sign-On.
- Федерация аккаунтов.
- Внутренний auth service, который должен быть совместим с внешними клиентами.

## ID Token

ID Token подтверждает факт аутентификации пользователя для конкретного клиента.

Обязательно проверять:

- `iss` - ожидаемый issuer.
- `aud` - client id текущего приложения.
- `exp` и `iat`.
- `nonce`, если он был отправлен в auth request.
- Подпись через JWKS.
- `azp`, если токен выпущен для нескольких audience.

## Access Token и ID Token

ID Token не должен использоваться для доступа к API. Для API используется access token.

Разделение:

- ID Token - для клиента, чтобы понять, кто вошел.
- Access Token - для resource server, чтобы проверить доступ к API.
- Refresh Token - для получения новых access tokens.

## Discovery

OIDC discovery обычно доступен по пути:

```text
/.well-known/openid-configuration
```

В metadata должны быть:

- issuer
- authorization_endpoint
- token_endpoint
- userinfo_endpoint
- jwks_uri
- supported scopes
- supported response types
- supported signing algorithms

## UserInfo

UserInfo endpoint возвращает claims о пользователе. Не стоит помещать в ID Token слишком много персональных данных, если клиенту они не нужны.

## Практики

- Использовать Authorization Code + PKCE.
- Устанавливать короткий lifetime access token.
- Ротировать refresh tokens.
- Поддерживать key rollover через JWKS.
- Логировать security events без записи secrets и полных токенов.
