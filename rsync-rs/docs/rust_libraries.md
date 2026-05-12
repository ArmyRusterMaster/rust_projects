# Rust-библиотеки для реализации

## CLI

- `clap` - аргументы командной строки, subcommands, help, shell completions.
- `clap_complete` - генерация completions для bash/zsh/fish.

## Обход файловой системы

- `walkdir` - простой рекурсивный обход директорий.
- `ignore` - обход с поддержкой gitignore-подобных правил.
- `globset` - быстрые glob-правила для include/exclude.
- `camino` - UTF-8 paths, если хочется строгой работы с путями.

## Копирование и ввод-вывод

- `fs_extra` - готовые операции копирования директорий, если нужен быстрый прототип.
- `tempfile` - временные файлы и директории для atomic write и тестов.
- `same-file` - проверка, не указывают ли два пути на один и тот же файл.

## Хеширование

- `blake3` - быстрый современный хеш для checksum-режима.
- `sha2` - SHA-256, если нужен более привычный формат.
- `crc32fast` - быстрая контрольная сумма, если нужна не криптографическая проверка.

## Конфигурация и форматы вывода

- `serde` - сериализация структур.
- `serde_json` - JSON output и отчеты.
- `toml` - конфигурация `rsync-rs.toml`.
- `schemars` - JSON Schema для стабильного формата отчетов или конфига.

## Логи, прогресс, ошибки

- `tracing` - структурированные логи.
- `tracing-subscriber` - настройка вывода логов.
- `indicatif` - progress bars.
- `thiserror` - типизированные ошибки в библиотечном коде.
- `anyhow` - удобный error handling в бинарнике.

## Параллелизм

- `rayon` - простой CPU-bound параллелизм, например для хеширования.
- `tokio` - async runtime, если появятся remote targets или сетевые операции.
- `crossbeam-channel` - очереди задач для worker pipeline.

## Тестирование

- `assert_cmd` - тестирование CLI.
- `predicates` - проверки stdout/stderr.
- `insta` - snapshot-тесты вывода.
- `proptest` - property-based тесты.
- `tempfile` - временные директории для integration-тестов.

## TUI, если понадобится

- `ratatui` - терминальный интерфейс.
- `crossterm` - работа с терминалом и событиями.

## Рекомендуемый минимальный набор для MVP

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
walkdir = "2"
ignore = "0.4"
globset = "0.4"
blake3 = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
tracing = "0.1"
tracing-subscriber = "0.3"
indicatif = "0.17"
thiserror = "2"
anyhow = "1"
tempfile = "3"

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
insta = "1"
proptest = "1"
```
