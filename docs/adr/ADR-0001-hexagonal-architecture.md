# ADR-0001 — Hexagonal architecture with ports & adapters

| Field | Value |
| :--- | :--- |
| **Status** | Accepted |
| **Date** | 2026-07-11 |
| **Deciders** | Mohamed Gamil |
| **Relates to** | [02-architecture.md](../02-architecture.md), [CONVENTIONS §2](../../delivery/CONVENTIONS.md) |

## Context

Git Purge must run as both a CLI tool and a Tauri desktop application, sharing
100 % of the domain logic. A traditional layered architecture would let the CLI
and UI drift: one might shell out to `git`, the other use `gix` in-process, and
neither would notice the inconsistency until production. We need a design that
makes "shared behaviour" a compile-time guarantee, not a convention.

Additionally, the git backend story itself is hybrid (gix for reads, git2 for
mutations — see ADR-0002), the credential store has multiple backends (OS
keychain, encrypted file), and the history database (SQLite) should never leak
into the CLI or UI crates. Every external concern must be swappable for tests
(deterministic fakes) and for future extensibility (new VCS providers, new
storage engines).

## Decision

Adopt **hexagonal architecture (ports & adapters)**:

1. **`gitpurge-core`** (library crate) owns all domain types, business rules,
   and service orchestration. It defines **port traits** for every external
   concern: `GitBackend`, `SecretStore`, `HistoryStore`, `ReportSink`, `Clock`,
   `ProgressSink`.

2. **`gitpurge-cli`** and **`gitpurge-desktop`** are thin adapters. They parse
   input, construct the `Engine` with production adapters, call `Engine` methods,
   and render output. They contain **zero** git/DB/keychain logic.

3. **Architecture guard tests** in each adapter crate fail if `gix`, `git2`,
   `rusqlite`, or `keyring` appear in their dependency tree. `deny.toml` bans
   mirror these. This enforces R6 structurally.

4. The `Engine` facade is `Send + Sync` (asserted at compile time) so the Tauri
   backend can share it across async command handlers.

## Consequences

- **Positive**: CLI and desktop are behaviourally identical by construction.
  Tests use in-memory fakes (no network, no disk, deterministic). Adding a new
  adapter (e.g., a language server, a web API) requires no core changes.

- **Negative**: More boilerplate (trait definitions + fake impls). Every new
  external concern needs a new port trait, even if there's only one adapter.

- **Risk**: Port traits may become too fine-grained or too coarse. Mitigated by
  reviewing the trait surface at each phase boundary.

## Alternatives considered

| Alternative | Why rejected |
| :--- | :--- |
| Direct library calls (no traits) | Prevents test fakes; couples CLI/UI to git library internals. |
| Microservice / IPC split | Over-engineering for a local tool; latency penalty; deployment complexity. |
| Shared dynamic library (.so/.dll) | Complicates cross-compilation and Tauri bundling; Rust's static linking is simpler. |
