# Git Purge — Canonical Conventions & Decisions

> **This file is the single source of truth.** Every spec doc, delivery doc, and
> implementation agent MUST conform to the names, versions, and decisions here.
> If another document conflicts with this file, this file wins — open a PR to fix
> the other document. Changes to canonical decisions require an ADR (see
> [`../docs/adr/`](../docs/adr/)).

---

## 1. Product identity

| Item | Value |
| :--- | :--- |
| Product name | **Git Purge** |
| Tagline | "Safely purge stale branches — with a net under every operation." |
| CLI binary name | `git-purge` (also usable as the git subcommand `git purge` when on `PATH`) |
| Desktop app name | **Git Purge** (product), bundle id `com.gitpurge.desktop` |
| GitHub repo | `MohamedGamil/git-purge` |
| License | **Apache-2.0** |
| Primary language | Rust (core + CLI + Tauri backend) |
| UI stack | Vue 3 + Vite + TypeScript |

## 2. Canonical crate & package names

| Path | Name | Kind | Purpose |
| :--- | :--- | :--- | :--- |
| `crates/gitpurge-core` | `gitpurge-core` | lib | All domain logic. No CLI/UI deps. The shared brain. |
| `crates/gitpurge-cli` | `gitpurge-cli` (bin `git-purge`) | bin | Thin CLI over `gitpurge-core`. |
| `apps/desktop/src-tauri` | `gitpurge-desktop` | bin | Tauri v2 backend; depends on `gitpurge-core`. |
| `apps/desktop` | `@gitpurge/desktop` | npm | Vue 3 frontend for the Tauri app. |

**Rule:** the UI and CLI are *both* thin adapters. They contain **zero** git logic.
All behavior lives in `gitpurge-core` and is exercised identically by both. This is
non-negotiable and is what Requirement 6 (shared abstractions) demands.

## 3. Toolchain & version targets

| Tool | Pinned target | Notes |
| :--- | :--- | :--- |
| Rust | stable, MSRV **1.88** | Pin via `rust-toolchain.toml`. Deps require ≥1.88 (time, icu, home). |
| Rust edition | 2021 | |
| Tauri | **v2.x** | Not v1. Use `tauri = "2"`, `@tauri-apps/api@^2`. |
| Node | **20 LTS** | For building the frontend only; not a runtime dep. |
| Vue | **3.4+** | Composition API + `<script setup>` only. |
| Vite | **5.x** | |
| TypeScript | **5.x** | `strict: true`. |
| Pinia | **2.x** | State management. |
| Vue Router | **4.x** | |
| pnpm | **9.x** | Package manager for the frontend workspace. |

## 4. Git engine strategy (ADR-0002)

Hybrid behind a single trait `GitBackend` in `gitpurge-core::git`:

- **`gix` (gitoxide)** — primary, for reads: ref enumeration, commit walking,
  diffs, object access, merge-base/ancestry checks. Pure Rust → static builds,
  fast, no C toolchain.
- **`git2` (libgit2)** — fallback for operations gix does not yet cover completely
  or ergonomically: pushes/deletes to remotes, credential callbacks, some fetch
  auth flows.
- **System `git` shell-out** — last-resort adapter (`ShellGitBackend`) used only
  when a needed operation is unavailable in both libs, or for parity testing.
  Never required for the "single binary, zero-setup" happy path.

The trait lets us swap/compose backends and add new VCS providers later
(Requirement 6). Callers depend on the trait, never on `gix`/`git2` directly.

## 5. Storage & data locations

Resolved via the `directories` crate (XDG on Linux, Known Folders on Windows,
Standard Dirs on macOS). Never hardcode paths (the old bash scripts hardcoded
`/home/mgamil/...` — Git Purge must not).

| Data | Location (logical) | Format |
| :--- | :--- | :--- |
| Config | `<config_dir>/git-purge/config.toml` | TOML |
| History / trends DB | `<data_dir>/git-purge/history.db` | SQLite (`rusqlite`, bundled) |
| Backups root (default) | `<data_dir>/git-purge/backups/` | See §6 |
| Per-repo state | keyed by canonical repo URL + local path hash | in SQLite |
| Logs | `<state_dir>/git-purge/logs/` | JSON lines |
| Secrets | OS keychain via `keyring`; encrypted file fallback | see auth spec |

Backups root is user-overridable per repo and globally (`config.toml` + CLI flag).

## 6. Backup model (ADR-0005) — the "minimal space" rule

- One **bare mirror** per source repo under `backups/<repo-id>.git` (like the old
  `backup_repos.sh`, but managed and space-shared).
- Each **backup snapshot** does NOT create a new clone. It writes the current refs
  into a namespaced backup ref inside the same bare repo:
  `refs/gitpurge/backups/<snapshot-id>/<original-ref-path>`.
  Because snapshots share the object database, N snapshots cost ~O(changed objects),
  not O(N × repo size). This is how we satisfy "minimal space consumption".
- A snapshot records metadata (SQLite + a `snapshot.json` manifest inside the repo):
  snapshot id, timestamp, source repo, trigger (manual / pre-delete / scheduled),
  and for every captured ref: branch name, tip commit SHA, commit count, upstream,
  merged/unmerged status at capture time.
- **Restore** reads a backup ref and recreates a branch **or** a tag (user's choice,
  Requirement 2/10), never force-overwriting an existing ref without explicit consent.
- **Auto-restore on failed delete:** any destructive op is wrapped so a failure
  triggers an offered restore from the just-created pre-op snapshot.

## 7. Safety model (must hold everywhere — see `docs/11-safety-model.md`)

1. **Dry-run is the default** for every mutating command. Mutations require an
   explicit `--execute` (CLI) / confirmation action (UI).
2. **Confirmation** required before execution; destructive/unmerged ops require a
   stronger confirmation (typed token or explicit toggle).
3. **Protected refs** are never touched: `main`, `master`, `develop`, `staging`,
   `production`, `HEAD`, plus a user-configurable protected list and glob patterns.
4. **Tags are never deleted** by branch operations (explicit guard).
5. **Backup-before-destroy**: a pre-op snapshot is created (and verified) before any
   delete/archive unless the user explicitly opts out with `--no-backup`.
6. Every mutating op is **logged** (append-only journal) for audit and undo.

## 8. Domain vocabulary (use these exact terms everywhere)

- **Repository / Repo** — a tracked git repo (local path and/or remote URL).
- **Ref** — any git reference; **Branch** (local/remote), **Tag**.
- **Classification** — computed facets of a branch: `merged`/`unmerged`,
  `local`/`remote`, `stale`/`active` (by age threshold), `protected`,
  `standard`/`non-standard` (naming policy), `ahead`/`behind` counts.
- **Policy** — user-configured thresholds & rules (age, naming regex, protected list).
- **Snapshot** — a point-in-time backup of refs (see §6).
- **Restore Point** — a snapshot from which refs can be restored.
- **Action** — a mutating operation: `delete`, `archive`, `restore`.
- **Plan** — the resolved set of Actions a command *would* take (dry-run output).
- **Run / Report** — a recorded execution and its metrics; feeds trend history.

## 9. CLI command surface (canonical — full detail in `docs/05-cli-spec.md`)

```
git-purge repo    add|list|remove|show          # manage tracked repos
git-purge scan    [--repo <id>] [--json]         # classify branches -> Plan-less audit
git-purge plan    [filters]                      # show what delete/archive WOULD do (dry-run)
git-purge backup  create|list|show|verify|prune  # snapshot management
git-purge delete  [filters] [--execute]          # delete stale/merged branches (safe)
git-purge archive [filters] [--execute]          # merge unmerged into legacy branch
git-purge restore <snapshot> <ref> [--as-tag]    # restore branch or tag
git-purge diff    <refA> <refB>                  # compare branches
git-purge show    <ref> [<path>]                 # view repo/file content at a ref/commit
git-purge report  [--format md|json|html]        # generate audit + trend reports
git-purge history [--repo <id>]                  # trend history
git-purge auth    add|list|remove|test           # credential management
git-purge ui                                     # launch desktop UI if installed
git-purge install-cli [--user|--system]          # put git-purge on PATH
git-purge completions <shell>                    # shell completions
```

Global flags: `--repo`, `--json`, `--no-color`, `--config <path>`, `--yes`,
`--execute`, `-v/-vv`, `--quiet`. Default filters mirror the old scripts:
`--age "1 year ago"`, `--merged`, `--include-unmerged`, `--exclude <glob,...>`.

## 10. Tauri IPC command surface (canonical — full detail in `docs/06-ui-spec.md`)

Every UI capability is a `#[tauri::command]` that calls `gitpurge-core`. Names mirror
the CLI verbs: `repo_list`, `scan`, `plan`, `backup_create`, `delete_branches`,
`archive_branches`, `restore`, `diff`, `show_tree`, `report_generate`,
`history_get`, `auth_*`. Long-running ops stream progress via Tauri events
(`gitpurge://progress`) and are cancellable.

## 11. Error handling & result types

- `gitpurge-core` uses `thiserror` for a typed `GitPurgeError` enum; public fns
  return `Result<T, GitPurgeError>`.
- CLI maps errors to exit codes + `miette`/`anstyle` rendered messages.
- Tauri commands return `Result<T, SerializableError>` where `SerializableError`
  is a serde-friendly projection of `GitPurgeError` (code + message + hint).
- No `unwrap()`/`expect()` in library code paths reachable at runtime (lint-enforced).

## 12. Coding standards

- `cargo fmt` + `cargo clippy -- -D warnings` are CI gates.
- `#![deny(unsafe_code)]` in `gitpurge-core`, with a single documented
  `#[allow(unsafe_code)]` exemption for libgit2 global timeout configuration
  ([ADR-0006](../docs/adr/ADR-0006-unsafe-libgit2-timeouts.md)).
- `#![forbid(unsafe_code)]` in `gitpurge-cli` unless an ADR justifies an exception.
- Public items documented with `///`; crate-level docs in `lib.rs`.
- Frontend: ESLint + Prettier + `vue-tsc --noEmit` are CI gates.
- Frontend layout, typography, components, and color systems must adhere to [DESIGN.md](../DESIGN.md).
- Conventional Commits for messages; `feat|fix|docs|chore|refactor|test|ci|build`.


## 13. Testing bar (see `docs/12-testing-strategy.md`)

- Unit tests colocated (`#[cfg(test)]`), integration tests in `crates/*/tests/`.
- Deterministic **fixture repos** built programmatically (a `testkit` module) so
  tests never depend on network or the user's machine.
- CLI tested with `assert_cmd` + `insta` snapshots.
- UI unit-tested with Vitest; e2e with `tauri-driver` + WebDriver.
- Coverage target: **≥ 80%** line coverage on `gitpurge-core`; every safety
  invariant in §7 has a dedicated regression test.

## 14. Distribution (see `docs/13-distribution-and-ci.md`)

- **Primary:** portable tarball per platform containing the `git-purge` binary
  (self-contained, no runtime deps) + `install-cli` helper. Zero-setup.
- Tauri bundles for all targets: `.deb`, `.rpm`, `.AppImage` (Linux),
  `.msi`/NSIS `.exe` (Windows), `.dmg`/`.app` (macOS).
- `release.yml` triggers on tag `v*`, builds the matrix, and attaches all
  artifacts + checksums to a GitHub Release. `ci.yml` runs on push/PR.

## 15. Document status legend (use in every doc header)

`Status: Draft | Reviewed | Approved` · `Owner: <role>` · `Last-updated: <date>` ·
`Related: <links>`. All dates ISO-8601. "Today" for authoring is **2026-07-11**.
