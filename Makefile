# Makefile for Coriolis
# Provides convenient shortcuts for common cargo commands

.PHONY: all build release test clean install uninstall run help static dev fmt clippy doc

# Default target
all: release

# Development build (fast, includes debug symbols)
build dev:
	cargo build

# Production build (optimized, no debug symbols)
release:
	cargo build --release

# Static binary for Linux (portable, no dependencies)
static:
	@echo "Building static binary with musl..."
	@if ! command -v musl-gcc >/dev/null 2>&1; then \
		echo "Error: musl-tools not installed"; \
		exit 1; \
	fi
	@if ! rustup target list | grep -q "x86_64-unknown-linux-musl (installed)"; then \
		echo "Installing musl target..."; \
		rustup target add x86_64-unknown-linux-musl; \
	fi
	PKG_CONFIG_ALL_STATIC=1 cargo build --release --target x86_64-unknown-linux-musl
	@echo "Stripping symbols..."
	@strip target/x86_64-unknown-linux-musl/release/coriolis 2>/dev/null || true
	@echo ""
	@echo "Static binary: target/x86_64-unknown-linux-musl/release/coriolis"
	@echo "Size: $$(du -h target/x86_64-unknown-linux-musl/release/coriolis | cut -f1)"

# Run tests
test:
	cargo test

# Run tests with output
test-verbose:
	cargo test -- --nocapture

# Format code with rustfmt
fmt:
	cargo fmt

# Check formatting
fmt-check:
	cargo fmt -- --check

# Run clippy linter
clippy:
	cargo clippy -- -D warnings

# Generate documentation
doc:
	cargo doc --no-deps --open

# Clean build artifacts
clean:
	cargo clean
	rm -f coriolis.log

# Install to system (requires sudo/root)
install: release
	@echo "Installing coriolis to /usr/local/bin..."
	@install -m 755 target/release/coriolis /usr/local/bin/coriolis
	@echo "Installed successfully!"

# Install static binary to system
install-static: static
	@echo "Installing static coriolis to /usr/local/bin..."
	@install -m 755 target/x86_64-unknown-linux-musl/release/coriolis /usr/local/bin/coriolis
	@echo "Installed successfully!"

# Uninstall from system
uninstall:
	@echo "Removing coriolis from /usr/local/bin..."
	@rm -f /usr/local/bin/coriolis
	@echo "Uninstalled successfully!"

# Run the application (requires a file argument)
run:
	@if [ -z "$(FILE)" ]; then \
		echo "Usage: make run FILE=path/to/file.nc"; \
		exit 1; \
	fi
	cargo run --release -- $(FILE)

# Run in development mode
run-dev:
	@if [ -z "$(FILE)" ]; then \
		echo "Usage: make run-dev FILE=path/to/file.nc"; \
		exit 1; \
	fi
	cargo run -- $(FILE)

# Check everything (fmt, clippy, test, build)
check: fmt-check clippy test build
	@echo "All checks passed!"

# Development workflow: watch and rebuild on changes (requires cargo-watch)
watch:
	@if ! command -v cargo-watch >/dev/null 2>&1; then \
		echo "cargo-watch not installed"; \
		echo "Install with: cargo install cargo-watch"; \
		exit 1; \
	fi
	cargo watch -x build

# Show help
help:
	@echo "Coriolis - Makefile targets:"
	@echo ""
	@echo "  make build        - Development build (fast)"
	@echo "  make release      - Production build (optimized)"
	@echo "  make static       - Static binary (no dependencies)"
	@echo "  make test         - Run tests"
	@echo "  make fmt          - Format code"
	@echo "  make clippy       - Run linter"
	@echo "  make doc          - Generate documentation"
	@echo "  make clean        - Remove build artifacts"
	@echo "  make install      - Install to /usr/local/bin (requires sudo)"
	@echo "  make install-static - Install static binary (requires sudo)"
	@echo "  make uninstall    - Remove from /usr/local/bin (requires sudo)"
	@echo "  make run FILE=... - Run with file"
	@echo "  make check        - Run all checks (fmt, clippy, test)"
	@echo "  make help         - Show this help"
	@echo ""
	@echo "Examples:"
	@echo "  make static                    # Build portable binary"
	@echo "  sudo make install-static       # Install portable binary"
	@echo "  make run FILE=data.nc          # Run with data file"
	@echo "  make check                     # Run all quality checks"
