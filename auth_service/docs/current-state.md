# Current State

## Что уже реализовано

- Базовая архитектура `domain -> ports -> application -> adapters`.
- Рабочий `InMemoryAuthRepository`.
- Рабочий `SqliteAuthRepository` на `sqlx` под feature `sqlite`.
- Инициализация SQLite-схемы через `SqliteAuthRepository::initialize_schema()`.
- Основные auth use-cases:
  - регистрация;
  - login;
  - refresh с rotation;
  - logout;
  - introspection access token.
- Тесты базовых auth flow для in-memory и SQLite.

## Ограничения текущего состояния

- Внешние DTO пока не отделены от storage-моделей: `User` содержит `password_hash`.
- Токены и пароли еще не обернуты в redacted secret-типы.
- Rotation refresh token реализована неатомарно (несколько операций вместо одной транзакционной команды).
- Нет HTTP API-слоя, только application/service слой.
- Нет rate limiting, MFA, password reset и полноценной policy-based authorization.

## Что стоит сделать следующим

1. Разделить внутренние и внешние модели (`UserRecord`/`PublicUser`) и убрать утечку `password_hash` наружу.
2. Перевести чувствительные поля на `secrecy::SecretString` и убрать `Debug` для секретов.
3. Добавить транзакционный метод ротации refresh token в repository contract.
4. Добавить HTTP слой (`axum`) и централизованный маппинг ошибок в API-ответы.
5. Добавить миграции для SQLite и интеграционные тесты с фиксированным контрактом хранилища.
6. Реализовать production adapters: `postgres` и `surrealdb`.
