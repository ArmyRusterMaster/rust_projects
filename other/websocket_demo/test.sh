#!/bin/bash

echo "Starting WebSocket server with tracing logging..."
RUST_LOG=debug cargo run -- server &
SERVER_PID=$!

sleep 2

echo "Running client test with tracing..."
RUST_LOG=debug cargo run -- client

echo "Stopping server..."
kill $SERVER_PID

echo "Test completed!"
