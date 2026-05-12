# Static Linking

Проект подготовлен к статической линковке через `musl`.

## Что настроено

- `./.cargo/config.toml`:
  - включен `crt-static` для `x86_64-unknown-linux-musl` и `aarch64-unknown-linux-musl`.
- Для `sqlx` используется `runtime-tokio-rustls` (без OpenSSL), это упрощает полностью статическую сборку.

## Сборка

1. Убедиться, что установлен target `musl` (способ зависит от вашего toolchain-менеджера, часто это `rustup target add ...`).

2. Собрать release:

```sh
cargo build --release --target x86_64-unknown-linux-musl
```

Бинарник будет в:

```sh
target/x86_64-unknown-linux-musl/release/auth_service
```

## Проверка

```sh
file target/x86_64-unknown-linux-musl/release/auth_service
ldd target/x86_64-unknown-linux-musl/release/auth_service
```

Для полностью статического бинарника `ldd` обычно пишет, что это не динамический executable.
