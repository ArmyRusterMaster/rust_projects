# Общие фичи аналогов rsync

Цель проекта `rsync-rs`: сделать практичную CLI-утилиту для синхронизации файлов, которую можно показать как портфолио-проект. Ниже собраны общие функции, которые часто встречаются у `rsync` и его сильных аналогов: `rclone`, `Syncthing`, `Unison`, `restic`, `BorgBackup`.

## Инструменты-аналоги

| Инструмент | Основной сценарий | Что важно изучить |
| --- | --- | --- |
| `rsync` | Односторонняя синхронизация файлов и директорий | delta-transfer, archive mode, dry-run, exclude/include |
| `rclone` | Синхронизация локальных файлов и облачных хранилищ | remotes, copy/sync/check, фильтры, bandwidth limit |
| `Syncthing` | Непрерывная P2P-синхронизация между устройствами | decentralized model, device trust, versioning |
| `Unison` | Двусторонняя синхронизация директорий | conflict detection, bidirectional sync |
| `restic` | Резервное копирование | snapshots, encryption, deduplication |
| `BorgBackup` | Эффективные бэкапы | deduplication, compression, authenticated encryption |

## Общий набор фич

1. Синхронизация директорий

   Базовая функция: привести целевую директорию к состоянию исходной. Для MVP достаточно локального режима:

   ```bash
   rsync-rs sync ./src ./dst
   ```

2. Копирование только измененных файлов

   Типовой подход: сравнивать размер, время изменения и при необходимости хеш содержимого. Для портфолио полезно сделать два режима:

   ```bash
   rsync-rs sync ./src ./dst --quick-check
   rsync-rs sync ./src ./dst --checksum
   ```

3. Dry-run

   Показывает, что будет сделано, но ничего не меняет:

   ```bash
   rsync-rs sync ./src ./dst --dry-run
   ```

4. Include/exclude правила

   Важны для реального применения:

   ```bash
   rsync-rs sync ./src ./dst --exclude target --exclude .git
   ```

5. Удаление лишних файлов в целевой директории

   Опасная, но полезная функция. Должна быть явной:

   ```bash
   rsync-rs sync ./src ./dst --delete
   ```

6. Сохранение метаданных

   В полном режиме стоит сохранять права, время изменения и симлинки, где это поддерживается системой.

7. Прогресс и статистика

   Нужны для UX:

   ```text
   files scanned: 1284
   files copied: 37
   bytes copied: 184.2 MB
   skipped: 1247
   elapsed: 3.2s
   ```

8. Проверка результата

   Отдельная команда для сравнения источника и цели:

   ```bash
   rsync-rs check ./src ./dst
   ```

9. Конфигурация

   Удобный формат для повторяемых задач:

   ```toml
   [job.backup]
   source = "./project"
   target = "/sdcard/backups/project"
   exclude = ["target", ".git"]
   delete = false
   ```

10. Машиночитаемый вывод

    Для DevOps и автоматизации:

    ```bash
    rsync-rs sync ./src ./dst --output json
    ```

## Источники

- rsync documentation: https://rsync.samba.org/documentation.html
- rsync man page: https://download.samba.org/pub/rsync/rsync.1
- rclone documentation: https://rclone.org/docs/
- Syncthing documentation: https://docs.syncthing.net/
- Unison manual: https://github.com/bcpierce00/unison/wiki/Documentation
- restic documentation: https://restic.readthedocs.io/
- BorgBackup documentation: https://borgbackup.readthedocs.io/
