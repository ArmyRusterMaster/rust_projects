# План реализации rsync-rs

Дата: 2026-05-12.

## Цель

Собрать тестируемый MVP локального `rsync`-подобного инструмента, который можно расширять без переписывания ядра.

## Архитектура

1. `scanner` - обходит дерево файлов, применяет include/exclude правила, собирает метаданные и checksum при необходимости.
2. `planner` - сравнивает source/target snapshots и строит детерминированный список операций.
3. `executor` - выполняет план, поддерживает dry-run, atomic copy, delete и trash.
4. `reporter` - печатает table/json/ndjson и формирует JSON-отчет.
5. `cli` - только парсит аргументы, валидирует сценарий и вызывает библиотечное ядро.

## Реализуемый MVP

- Команды `plan`, `sync`, `check`.
- Локальная синхронизация директорий.
- `--dry-run` для безопасной проверки.
- `--delete` для удаления лишнего из target.
- `--trash <dir>` для карантина удаляемых файлов и конфликтов.
- `--exclude <pattern>` и `--include <pattern>`.
- `--checksum` для сравнения содержимого через BLAKE3.
- Быстрый режим по умолчанию: размер и modified time.
- Table, JSON и NDJSON вывод через `--output`.
- JSON-отчет через `--report`.
- Итоговая статистика и стабильные exit codes.
- Atomic copy через временный файл и rename.
- Сохранение прав и modified time для файлов.
- Глобальный allocator `mimalloc` через feature `mimalloc-allocator`.

## Профили сборки

- `dev` и `test`: быстрые проверки, минимум оптимизаций, incremental build, много codegen units.
- `release`: быстрый runtime, `opt-level = 3`, ThinLTO, один codegen unit, `panic = abort`, strip symbols.

## Тестирование

- Unit-тесты фильтров и planner.
- Integration-тесты синхронизации через временные директории.
- Проверки: `cargo fmt --check`, `cargo check`, `cargo test`.

## Следующие расширения

- Конфиг `rsync-rs.toml` и команда `run`.
- Progress bar и structured logs.
- Параллельное хеширование/копирование.
- Resume для больших файлов.
- Snapshot/backup режим.
- Shell completions и man page.
