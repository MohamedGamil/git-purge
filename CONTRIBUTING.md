# Contributing to Git Purge

Thank you for your interest in contributing to Git Purge! This document provides
guidelines and information for contributors.

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md).
By participating, you are expected to uphold this code.

## Getting Started

### Prerequisites

- **Rust** stable toolchain (MSRV 1.88) — install via [rustup](https://rustup.rs/)
- **Git** 2.30+
- **Node.js** 20 LTS (for desktop frontend development only)

### Setup

```bash
# Clone the repository
git clone https://github.com/MohamedGamil/git-purge.git
cd git-purge

# Build the workspace (excluding desktop until Tauri deps are installed)
cargo build --workspace --exclude gitpurge-desktop

# Run tests
cargo nextest run --workspace --exclude gitpurge-desktop

# Run the CLI
cargo run --bin git-purge -- --help
```

### Project Structure

```
git-purge/
├── crates/
│   ├── gitpurge-core/   # Shared domain library (the "brain")
│   └── gitpurge-cli/    # CLI binary (thin adapter)
├── apps/
│   └── desktop/         # Tauri v2 desktop app (thin adapter)
├── docs/                # Specification documents
├── delivery/            # Phase delivery plans and conventions
└── .github/workflows/   # CI configuration
```

### Architecture Rules

1. **gitpurge-core** is the only crate with domain logic. It must never depend on
   CLI or UI concerns.
2. **gitpurge-cli** and **gitpurge-desktop** are thin adapters. They must never
   contain git operations, database access, or business logic directly.
3. All external concerns (git, auth, DB, reporting) go through port traits defined
   in `gitpurge-core`.

For full details, see [CONVENTIONS.md](delivery/CONVENTIONS.md) and
[02-architecture.md](docs/02-architecture.md).

## Development Workflow

### Branching

- `main` — always releasable
- Feature branches: `feat/<short-description>`
- Bug fixes: `fix/<short-description>`

### Before Submitting a PR

1. **Format**: `cargo fmt --all`
2. **Lint**: `cargo clippy --workspace --exclude gitpurge-desktop --all-targets -- -D warnings`
3. **Test**: `cargo nextest run --workspace --exclude gitpurge-desktop`
4. **Commit messages**: follow [Conventional Commits](https://www.conventionalcommits.org/)

### Commit Message Format

```
<type>(<scope>): <short summary>

<optional body>

<optional footer>
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`, `ci`

Scopes: `core`, `cli`, `desktop`, `docs`, `ci`

## Reporting Issues

- Use GitHub Issues
- Include: OS, Rust version, steps to reproduce, expected vs actual behavior
- For security vulnerabilities, see [SECURITY.md](SECURITY.md)

## License

By contributing, you agree that your contributions will be licensed under the
[Apache License 2.0](LICENSE).
