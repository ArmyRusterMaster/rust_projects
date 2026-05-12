# Sessions And Cookies

Для браузерных приложений серверная сессия в `HttpOnly` cookie часто безопаснее, чем хранение bearer token в JavaScript.

## Cookie flags

Рекомендуемые флаги:

```text
HttpOnly
Secure
SameSite=Lax или SameSite=Strict
Path=/
```

Для cross-site сценариев может потребоваться `SameSite=None; Secure`, но это повышает требования к CSRF-защите.

## CSRF

Если браузер автоматически отправляет cookie, нужна CSRF-защита.

Подходы:

- Synchronizer token.
- Double-submit cookie.
- Проверка Origin и Referer для state-changing запросов.
- SameSite=Lax или Strict там, где это не ломает продукт.

## Session storage

Серверная сессия должна храниться в надежном backend:

- PostgreSQL - если важна долговечность и транзакционность.
- Redis - если важны скорость, TTL и простое удаление.
- Гибрид: PostgreSQL для audit/device records, Redis для горячих сессий.

## Logout

Logout должен:

- Удалять серверную сессию.
- Очищать cookie.
- Отзывать refresh token family, если сессия связана с токенами.
- Записывать событие в audit log.

## Защита сессий

- Ротировать session id после login и privilege escalation.
- Ограничивать lifetime idle и absolute.
- Хранить хеш session id, а не сырой идентификатор.
- Добавлять device/session management для пользователя.
- Отзывать все сессии при смене пароля или компрометации аккаунта.
