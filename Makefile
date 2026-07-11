# Git Purge — Development Makefile
# Excludes gitpurge-desktop (Tauri app) by default as it requires Tauri system dependencies.

.PHONY: all build check test clippy fmt run clean ui-dev install-desktop-deps

# Default target
all: check test

# Build all workspace crates (excluding desktop)
build:
	cargo build --workspace --exclude gitpurge-desktop --all-features

# Check compilation of all workspace crates (excluding desktop)
check:
	cargo check --workspace --exclude gitpurge-desktop --all-features

# Run all workspace tests (excluding desktop)
test:
	cargo test --workspace --exclude gitpurge-desktop --all-features

# Run Clippy checks for code quality and lints
clippy:
	cargo clippy --workspace --exclude gitpurge-desktop --all-features -- -D warnings

# Format check code
fmt:
	cargo fmt --all -- --check

# Format code automatically
fmt-write:
	cargo fmt --all

# Install required Desktop OS packages for Tauri to run on Ubuntu (requires sudo)
install-desktop-deps:
	sudo apt-get update && sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.1-dev libsoup-3.0-dev

# Run the desktop UI web app in dev mode for testing the interface
ui-dev:
	pnpm --filter @gitpurge/desktop dev

# Run the CLI binary (usage: make run ARGS="--help")
run:
	cargo run --bin git-purge -- $(ARGS)

# Clean build artifacts
clean:
	cargo clean
