#!/bin/bash
set -e

echo "=== Building GlassForge Platform ==="

echo "[1/3] Building frontend..."
cd frontend && pnpm install && pnpm build
cd ..

echo "[2/3] Building scanner..."
cd scanner && swift build -c release
cd ..

echo "[3/3] Building platform binary..."
cargo build --release

echo ""
echo "Done. Outputs:"
echo "  Platform: ./target/release/glassforge"
echo "  Scanner:  ./scanner/.build/release/glassforge-scan"
