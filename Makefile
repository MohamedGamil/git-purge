# Git Purge — Development Makefile
# Excludes gitpurge-desktop (Tauri app) by default as it requires Tauri system dependencies.

.PHONY: all build check test test-desktop test-all clippy fmt run clean ui-dev install-desktop-deps desktop-dev desktop-build desktop-test desktop-test-rust build-all coverage

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

test-desktop:
	cargo test --workspace --package gitpurge-desktop --all-features

test-all: test test-desktop

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

# Run the desktop application in dev mode (requires Tauri dependencies)
desktop-dev:
	pnpm --filter @gitpurge/desktop tauri dev

# Build the desktop application package (requires Tauri dependencies)
desktop-build:
	pnpm --filter @gitpurge/desktop tauri build

# Run desktop UI frontend unit tests
desktop-test:
	pnpm --filter @gitpurge/desktop test

# Run desktop Rust backend unit/integration tests
desktop-test-rust:
	cargo test -p gitpurge-desktop

# Build all CLI and desktop applications
build-all:
	make build
	make desktop-build

HOST_TRIPLE := $(shell rustc -Vv | grep host | cut -f2 -d' ')
ifeq ($(OS),Windows_NT)
    BUNDLE_EXT := zip
else
    BUNDLE_EXT := tar.gz
endif

# Package/bundle the CLI for distribution
bundle-cli:
	cargo build --release -p gitpurge-cli
	bash ci/package-tarball.sh $(HOST_TRIPLE) $(BUNDLE_EXT)

# Build the desktop application bundle for distribution
bundle-desktop:
	pnpm --filter @gitpurge/desktop tauri build

# Build and package both CLI and desktop apps for distribution
bundle: bundle-cli bundle-desktop


# Run the CLI binary (usage: make run ARGS="--help")
run:
	cargo run --bin git-purge -- $(ARGS)

# Clean build artifacts
clean:
	cargo clean

# Run coverage checks locally using cargo-llvm-cov
coverage:
	cargo llvm-cov --workspace --exclude gitpurge-desktop --all-features --fail-under-lines 50
