# Killer features для rsync-rs

Эти функции могут выделить проект среди простых копировщиков файлов и показать работодателю инженерный уровень.

## 1. План изменений перед выполнением

Команда строит детальный план операций:

```bash
rsync-rs plan ./src ./dst
```

Вывод:

```text
COPY    src/main.rs        12.4 KB
UPDATE  README.md          changed
DELETE  old.log            target-only
SKIP    target/debug/app   excluded
```

Плюс: легко тестировать, удобно показывать на собеседовании.

## 2. Безопасный `--delete` через карантин

Вместо немедленного удаления файлы сначала переносятся в скрытую папку:

```bash
rsync-rs sync ./src ./dst --delete --trash .rsync-rs-trash
```

Плюс: снижает риск потери данных.

## 3. Resume для больших файлов

Если копирование большого файла оборвалось, следующий запуск продолжает работу или валидирует уже записанные блоки.

```bash
rsync-rs sync ./videos ./backup --resume
```

Плюс: сильная фича для реального мира.

## 4. Блочная дедупликация

Хранить повторяющиеся блоки один раз для backup-режима:

```bash
rsync-rs backup ./src ./repo
```

Плюс: показывает знание хеширования, chunking и storage design.

## 5. Snapshot-режим

Создавать снимки состояния:

```bash
rsync-rs snapshot create ./src ./backup-repo
rsync-rs snapshot list ./backup-repo
rsync-rs snapshot restore ./backup-repo latest ./restore
```

Плюс: мост между `rsync` и backup-инструментами уровня `restic`/`Borg`.

## 6. Проверяемые отчеты

После синхронизации сохранять отчет:

```bash
rsync-rs sync ./src ./dst --report report.json
```

В отчете: версии, параметры запуска, список операций, хеши файлов, ошибки, время выполнения.

## 7. TUI-режим

Интерактивный экран для просмотра плана:

```bash
rsync-rs tui ./src ./dst
```

Плюс: демонстрирует UX, async/event loop и аккуратную архитектуру.

## 8. Профили синхронизации

Запуск именованных задач из конфига:

```bash
rsync-rs run phone-backup
rsync-rs run work-project --dry-run
```

Плюс: утилита выглядит как готовый рабочий инструмент, а не учебный скрипт.
