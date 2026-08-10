#!/bin/bash
set -e

cd "$(dirname "$0")"

if [ -f "server/server" ]; then
    cd server
    SERVER_BIN="./server"
elif [ -f "server" ]; then
    SERVER_BIN="./server"
elif [ -f "server/target/release/server" ]; then
    cd server
    SERVER_BIN="./target/release/server"
else
    echo "server binary was not found."
    echo "Please build the Linux package first."
    exit 1
fi

chmod +x "$SERVER_BIN" 2>/dev/null || true

echo "========================================"
echo "XianYu Music Server"
echo "========================================"
echo "Work directory: $(pwd)"
echo "Press Ctrl+C to stop."
echo ""

exec "$SERVER_BIN"
