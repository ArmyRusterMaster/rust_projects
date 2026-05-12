# Implementation Plan

## Цель

Собрать auth service так, чтобы доменная логика не зависела от конкретной базы данных, HTTP framework или формата токенов. Основная ось расширения - ports and adapters.

## Архитектура

Слои:

- `domain` - типы пользователя, сессии, access token, refresh token, audit event.
- `ports` - traits для repository, password hashing, token generation и clock.
- `application` - use cases: register, login, refresh, logout, introspection.
- `adapters` - конкретные реализации ports.

## Persistence

`AuthRepository` является стабильным контрактом для хранилищ.

Текущие адаптеры:

- `InMemoryAuthRepository` - рабочая реализация для тестов, прототипа и локальной разработки.
- `SqliteAuthRepository` - расширяемая заготовка под feature `sqlite`.
- `PostgresAuthRepository` - расширяемая заготовка под feature `postgres`.
- `SurrealDbAuthRepository` - расширяемая заготовка под feature `surrealdb`.

Следующий шаг для SQL-адаптеров - подключить `sqlx`, добавить миграции и реализовать методы `AuthRepository` без изменения `application`.

## Security baseline

Реализовано:

- Argon2 для хеширования паролей.
- Случайные bearer tokens с 256-bit entropy по умолчанию.
- Хранение только SHA-256 hash токенов.
- Короткий lifetime access token.
- Refresh token rotation.
- Token family revocation при reuse.
- Server-side sessions.
- Audit events для основных auth-событий.
- Logout с отзывом сессии и refresh token family.

## Testing plan

Тесты должны покрывать:

- регистрацию пользователя;
- запрет слабого пароля;
- запрет дублирующего email;
- успешный login;
- отказ при неверном пароле;
- introspection access token;
- refresh token rotation;
- reuse detection старого refresh token;
- logout и неактивность access token после logout.

## Extension plan

При добавлении новой БД:

1. Создать модуль в `src/adapters/persistence`.
2. Реализовать `AuthRepository`.
3. Добавить feature в `Cargo.toml`.
4. Добавить integration tests на общий contract repository.
5. Не менять `application`, если контракт repository достаточен.
