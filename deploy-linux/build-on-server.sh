#!/bin/bash
set -e
echo "=== Installing Rust ==="
if ! command -v cargo &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "C:\Users\??/.cargo/env"
fi
echo "Rust version: "

echo "=== Building server ==="
cd /www/wwwroot/xymusic.example.com/server
cargo build --release 2>&1
echo "Build complete!"
