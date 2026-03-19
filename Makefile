.PHONY: dev build test clean scan

# Development: run backend + frontend concurrently
dev:
	@echo "Starting GlassForge dev environment..."
	@cd frontend && pnpm dev &
	@cargo watch -x run

# Production build: frontend first (rust-embed needs build/), then cargo
build:
	@echo "Building frontend..."
	@cd frontend && pnpm install && pnpm build
	@echo "Building scanner..."
	@cd scanner && swift build -c release
	@echo "Building platform..."
	@cargo build --release
	@echo "Done. Binary: ./target/release/glassforge"

# Run all tests
test:
	@cargo test
	@cd scanner && swift test
	@cd frontend && pnpm test 2>/dev/null || true

# Clean all build artifacts
clean:
	@cargo clean
	@cd scanner && swift package clean 2>/dev/null || true
	@rm -rf frontend/build frontend/node_modules/.vite

# Scan a target iOS repo
scan:
	@cd scanner && swift build -c release && .build/release/glassforge-scan analyze $(TARGET)
