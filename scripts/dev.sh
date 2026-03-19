#!/bin/bash
set -e

echo "Starting GlassForge development environment..."

# Start frontend dev server in background
cd frontend && pnpm install && pnpm dev &
FRONTEND_PID=$!

# Start Rust backend with auto-reload
cd .. && cargo watch -x run

# Cleanup
kill $FRONTEND_PID 2>/dev/null
