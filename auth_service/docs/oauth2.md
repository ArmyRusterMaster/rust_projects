# OAuth 2.0

OAuth 2.0 - это протокол делегированной авторизации. Он не доказывает личность пользователя сам по себе, а выдает клиенту право выполнить действие от имени владельца ресурса.

## Роли

- Resource Owner - пользователь или система, владеющая ресурсом.
- Client - приложение, которому нужен доступ.
- Authorization Server - сервис, который аутентифицирует пользователя и выпускает токены.
- Resource Server - API, которое принимает и проверяет access token.

## Рекомендуемые flows

### Authorization Code + PKCE

Основной flow для web, mobile, desktop и SPA-клиентов.

Требования:

- Всегда использовать PKCE.
- Redirect URI должен быть заранее зарегистрирован и сравниваться строго.
- Authorization code должен быть одноразовым и короткоживущим.
- State обязателен для защиты от CSRF.
- Nonce обязателен, если flow используется вместе с OpenID Connect.

### Client Credentials

Подходит для machine-to-machine взаимодействия.

Требования:

- Выдавать минимальные scopes.
- Разделять client id и secret по окружениям.
- Ротировать secrets.
- Не использовать этот flow для действий от имени пользователя.

### Device Authorization Grant

Подходит для CLI, TV и устройств без удобного браузера.

Требования:

- Ограничивать lifetime device/user code.
- Использовать polling interval.
- Защищаться от brute force user code.

## Flows, которых лучше избегать

- Implicit Flow - заменяется Authorization Code + PKCE.
- Resource Owner Password Credentials - допустим только для legacy-сценариев с полным доверием к клиенту.

## Scopes

Scopes должны описывать разрешения клиента, а не роли пользователя.

Хорошие примеры:

- `users:read`
- `users:write`
- `orders:read`
- `offline_access`

Плохие примеры:

- `admin`
- `superuser`
- `full_access`

## Частые ошибки

- Использовать OAuth 2.0 как login-протокол без OpenID Connect.
- Хранить access token в localStorage для браузерных приложений.
- Делать refresh tokens бессрочными.
- Не проверять audience, issuer и expiration.
- Принимать токены от любого authorization server.
- Смешивать scopes клиента и права пользователя.
