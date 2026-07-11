# Contributing to Git Purge

Thanks for helping build Git Purge! This guide covers local setup, the quality gates
you must pass, and how to open a good PR. The canonical rules live in
[`delivery/CONVENTIONS.md`](delivery/CONVENTIONS.md) and
[`delivery/AGENT_GUIDE.md`](delivery/AGENT_GUIDE.md) — **read `CONVENTIONS.md` first;
it overrides everything.**

## Golden rules (non-negotiable)

1. **Logic lives in `gitpurge-core`.** The CLI (`gitpurge-cli`) and the Tauri backend
   (`gitpurge-desktop`) are thin adapters — they translate user intent and render
   results. They contain **zero** git/DB/keychain logic and must **not** depend on
   `gix`, `git2`, or `rusqlite` directly.
2. **Dry-run is the default** for every mutating command. Mutations require an
   explicit `--execute` (CLI) or a confirmed action (UI).
3. **Backup before destroy.** No destructive operation runs without a prior _verified_
   backup snapshot (unless the user explicitly passes `--no-backup`). If you add a
   destructive path, you add the backup wrapper and a test that proves it.
4. **No secrets or PII in logs, errors, snapshots, or reports.** Ever.
5. **No hardcoded paths, repos, users, or ages.** Everything is config or arguments —
   resolve paths through the `directories` crate.

## Dev setup

### Rust (core + CLI, always required)

```bash
rustup toolchain install stable        # rust-toolchain.toml pins the exact version (1.82)
cargo install cargo-nextest cargo-deny cargo-insta
```

### Frontend (only if working on the desktop app)

```bash
corepack enable && corepack prepare pnpm@9 --activate
cd apps/desktop && pnpm install
```

You will also need your platform's Tauri v2 prerequisites (WebKitGTK on Linux,
WebView2 on Windows, Xcode CLT on macOS) — see `docs/13-distribution-and-ci.md`.

## The local gate (must all pass before you push)

```bash
# Rust
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run --all
cargo deny check

# Frontend (desktop tasks only)
pnpm -C apps/desktop lint
pnpm -C apps/desktop test
pnpm -C apps/desktop exec vue-tsc --noEmit
```

CI runs the same gate on Linux, macOS, and Windows. A red gate blocks merge.

## Commits & branches

- **Conventional Commits** for messages:
  `feat|fix|docs|chore|refactor|test|ci|build(scope): summary`.
- **Branch naming:** `feat/P<phase>-T<n>-<slug>` (e.g. `feat/P3-T2-delete-cmd`),
  following the phases in [`docs/ROADMAP.md`](docs/ROADMAP.md).

## PR checklist (paste into every PR)

```
- [ ] Conforms to CONVENTIONS.md (names, versions, layout, safety model)
- [ ] Logic lives in gitpurge-core; CLI/UI are thin
- [ ] Dry-run default preserved; destructive paths backed up first
- [ ] Unit tests + ≥1 feature test added; all gates green locally
- [ ] No secrets/PII in logs, errors, or fixtures
- [ ] Docs/ADR updated if decisions or behavior changed
- [ ] Requirement(s) R__ referenced; DoD row(s) satisfied
```

## Testing bar

- Unit tests colocated (`#[cfg(test)]`); integration tests in `crates/*/tests/`.
- Use the deterministic `testkit` fixture-repo builders — **no network or
  machine-specific state in tests.**
- CLI is snapshot-tested with `assert_cmd` + `insta`.
- Target ≥ 80% line coverage on `gitpurge-core`; every safety invariant has a
  dedicated regression test.

## When specs are ambiguous

Prefer the safest, simplest interpretation consistent with `CONVENTIONS.md` and the
safety model, implement it, and note the assumption in your PR. Escalate only true
forks (irreversible or user-preference decisions).
