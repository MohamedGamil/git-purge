# Git Purge — Development Makefile
# Excludes gitpurge-desktop (Tauri app) by default as it requires Tauri system dependencies.

.PHONY: all build check test clippy fmt run clean

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

# Run the CLI binary (usage: make run ARGS="--help")
run:
	cargo run --bin git-purge -- $(ARGS)

# Clean build artifacts
clean:
	cargo clean
