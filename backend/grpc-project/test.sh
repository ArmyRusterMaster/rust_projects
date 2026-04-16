#!/bin/bash
# Сборка проекта
cargo build --release

# Запуск сервера в фоновом режиме
./target/debug/server &
SERVER_PID=$!

# Даем серверу время на запуск
sleep 2

# Запуск клиента и сохранение вывода
OUTPUT=$(./target/debug/client)
echo "$OUTPUT"

# Проверка результата
if [[ $OUTPUT == *"Привет, Termux User!"* ]]; then
  echo "✅ Тест пройден успешно!"
else
  echo "❌ Тест провален."
fi

# Завершение работы сервера
kill $SERVER_PID
