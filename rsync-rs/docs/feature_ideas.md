# Какие фичи можно добавить

## MVP

- `sync <source> <target>` для локальных директорий.
- `--dry-run`.
- `--delete`.
- `--exclude` и `--include`.
- `--checksum`.
- `--verbose`.
- Итоговая статистика.
- Корректные exit codes.
- Unit-тесты для построения плана операций.
- Integration-тесты на временных директориях.

## Версия 0.2

- Конфиг `rsync-rs.toml`.
- Команда `plan`.
- Команда `check`.
- Вывод `table`, `json`, `ndjson`.
- Progress bar.
- Ограничение скорости: `--bwlimit`.
- Параллельное сканирование директорий.
- Параллельное копирование маленьких файлов.
- Логи через `tracing`.

## Версия 0.3

- Resume для частично скопированных файлов.
- Карантин для удаляемых файлов.
- Snapshot-режим.
- Сжатие для backup-режима.
- Шифрование backup-репозитория.
- Remote target через SSH/SFTP.
- Telegram/webhook уведомления после выполнения job.

## Версия 1.0

- Стабильный формат конфигурации.
- Стабильный JSON output.
- Документация с примерами.
- CI: test, fmt, clippy, release build.
- Release binaries для Linux x86_64, Linux aarch64 и Android/Termux.
- Man page и shell completions.

## Идеи для демонстрации работодателю

- В README показать сравнение с `rsync`: что уже есть, что не реализовано, какие trade-offs.
- Добавить gif/asciinema с dry-run и progress bar.
- Приложить benchmarks на 10k маленьких файлов и на один большой файл.
- Написать архитектурный документ: scanner, planner, executor, reporter.
- Подготовить тестовый набор в `examples/fixtures`.
