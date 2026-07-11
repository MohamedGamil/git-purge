# Phase 0 — Foundations & scaffolding

`Status: Draft` · `Owner: Delivery` · `Last-updated: 2026-07-11` ·
`Related: [ROADMAP](../docs/ROADMAP.md#p0--foundations--scaffolding-6-ed), [CONVENTIONS](CONVENTIONS.md), [architecture §2](../docs/02-architecture.md), [AGENT_GUIDE](AGENT_GUIDE.md), [DoD](DEFINITION_OF_DONE.md)`

## Goal

Stand up the Cargo workspace and the three canonical crates so that everything
compiles, all CI gates are green, and the **ports-and-adapters seam is real** before
any git logic is written. This phase produces no user-facing behavior; it produces
the scaffolding — typed errors, config loading, the port traits with in-memory fakes,
the `testkit` fixture-repo builder, and the architecture test that structurally
forbids `gix`/`git2`/`rusqlite` from leaking into the CLI/UI crates. Getting these
seams right here is what makes Requirement 6 (shared abstractions) true by
construction rather than by discipline.

**Milestone:** M0 — Foundations ready (workspace builds, CI green, core skeleton +
ports compile with fakes).

**Dependencies:** none (first phase).

## Tasks

| Task ID | Title | Description | Files (repo-relative) | Depends-on | ∥? | Est (ED) | Acceptance test |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| **P0-T1** | Workspace + three crates compiling | Create the Cargo workspace root with shared deps, pin the toolchain, and scaffold the three canonical crates (`gitpurge-core` lib, `gitpurge-cli` bin `git-purge`, `gitpurge-desktop` under `apps/desktop/src-tauri`). Empty `lib.rs`/`main.rs` that compile. Names/versions per [CONVENTIONS §2–§3](CONVENTIONS.md). | `Cargo.toml`, `rust-toolchain.toml`, `crates/gitpurge-core/{Cargo.toml,src/lib.rs}`, `crates/gitpurge-cli/{Cargo.toml,src/main.rs}`, `apps/desktop/src-tauri/{Cargo.toml,src/main.rs}` | — | no | 1 | `cargo build --workspace` succeeds; `rustc --version` matches MSRV 1.82; workspace lists all three members incl. `apps/desktop/src-tauri`. |
| **P0-T2** | Typed errors | `GitPurgeError` enum via `thiserror` in `gitpurge-core`, plus the serde-friendly `SerializableError` projection (code + message + hint) used by Tauri. `#![forbid(unsafe_code)]` set in core. Per [CONVENTIONS §11](CONVENTIONS.md). | `crates/gitpurge-core/src/error.rs`, `crates/gitpurge-core/src/lib.rs` | P0-T1 | yes | 0.5 | Unit test round-trips a `GitPurgeError` variant → `SerializableError` → JSON with stable `code`; `forbid(unsafe_code)` compiles. |
| **P0-T3** | Config load/save | `config.rs` loading/saving `config.toml` via `directories` (XDG/KnownFolders/StandardDirs — **no hardcoded paths**, [CONVENTIONS §5](CONVENTIONS.md)); `Config`/`Policy` skeleton structs from [domain model §5](../docs/03-domain-model.md). Defaults: age `"1 year ago"`, seeded immutable `well_known` protected list. | `crates/gitpurge-core/src/config.rs`, `crates/gitpurge-core/src/model/policy.rs` | P0-T1 | yes | 1 | Round-trip test: default `Config` → TOML → parse → equal; resolved config path uses `directories`, contains no `/home/...` literal; `ProtectionPolicy.well_known` always contains the six well-known names. |
| **P0-T4** | Ports as traits + in-memory fakes | Define every port trait from [architecture §3](../docs/02-architecture.md): `GitBackend`, `SecretStore`, `HistoryStore`, `ReportSink`, `Clock`, `ProgressSink`; each with an in-memory fake adapter. Skeleton `Engine` facade holding injected ports (signatures from [architecture §4](../docs/02-architecture.md), bodies `todo!()`/unimplemented but compiling). | `crates/gitpurge-core/src/git/mod.rs`, `crates/gitpurge-core/src/auth/mod.rs`, `crates/gitpurge-core/src/history/mod.rs`, `crates/gitpurge-core/src/report/mod.rs`, `crates/gitpurge-core/src/{clock.rs,progress.rs}`, `crates/gitpurge-core/src/testkit/fakes.rs`, `crates/gitpurge-core/src/lib.rs` | P0-T1, P0-T2 | partly | 1.5 | Test constructs an `Engine` wired entirely from fakes (fake `Clock`, `GitBackend`, etc.) and calls a trivial method; proves dependency-inversion. `Engine: Send + Sync` asserted. |
| **P0-T5** | testkit fixture-repo builder | `testkit` module (behind a `testkit` feature) that builds deterministic on-disk git repos in a temp dir with no network: seed commits with fixed authors/dates, create local/remote branches, merged/unmerged, protected names, tags. Provide named builders: `merged_repo`, `stale_repo`, `multi_remote_repo`, `naming_repo`. | `crates/gitpurge-core/src/testkit/mod.rs`, `crates/gitpurge-core/src/testkit/builders.rs`, `crates/gitpurge-core/Cargo.toml` (feature) | P0-T1 | yes | 1 | `merged_repo()` builds a repo on disk under a temp dir with a known merged and unmerged branch; two invocations at fixed clock yield byte-identical commit SHAs (determinism). |
| **P0-T6** | Architecture guard test + deny bans | Test in the CLI crate (and the Tauri crate) that fails if `gix`/`git2`/`rusqlite`/`keyring` appear in their dependency tree; mirror the ban in `deny.toml`. Enforces [architecture §1](../docs/02-architecture.md) / R6. | `crates/gitpurge-cli/tests/architecture.rs`, `deny.toml` | P0-T1, P0-T4 | yes | 0.5 | Test passes today; adding `git2` to `gitpurge-cli/Cargo.toml` makes it (and `cargo deny check bans`) fail — verifies **R6** structurally. |
| **P0-T7** | CI gate (`ci.yml`) | GitHub Actions workflow running the local gate on push/PR: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo nextest run --all`, `cargo deny check`. Caching + MSRV pin. Consistent with [CONVENTIONS §12–§14](CONVENTIONS.md). | `.github/workflows/ci.yml` | P0-T1 | yes | 0.5 | Opening a PR triggers `ci.yml`; all four jobs green on the scaffold; a `clippy` warning fails the job. |

Total ≈ 6 ED.

## Exit criteria

- `cargo build --workspace`, `cargo clippy --all-targets -- -D warnings`, and
  `cargo nextest run --all` are all green in CI (`ci.yml`). (ROADMAP P0 exit.)
- All ports exist as traits with in-memory fakes; an `Engine` can be assembled purely
  from fakes — the fakes prove the layering.
- The architecture guard test forbids `gix`/`git2`/`rusqlite` in the CLI/UI crates and
  is enforced by both a test and `cargo-deny` bans.
- `testkit` builds deterministic fixture repos with no network access.

### Requirements & safety invariants satisfied

- **R6** (shared abstractions + extensibility): ports as traits + architecture guard
  test (P0-T4, P0-T6). Note: the "second `GitBackend` swaps in without facade change"
  proof lands in P1; the seam is established here.
- **R8** (testing mandatory): `nextest` + `testkit` foundation (P0-T5, P0-T7).
- Safety invariants (`SAFE-01..07`) are not yet exercised by behavior; P0 only creates
  the seams (dry-run default, protection policy shape) that later phases test.

## Risks & open questions

- **`gitpurge-desktop` as a workspace member vs. Tauri tooling expectations.** Building
  the Tauri crate in plain `cargo build` may need a stub frontend; confirm whether P0
  builds `src-tauri` with a `--no-default-features` / dev shim or defers full build to P4.
- **`directories` on headless CI** may resolve to unexpected dirs; tests must inject a
  config root override rather than touch the real user dirs.
- **Fake determinism vs. real git object hashing** — fixtures must fix author/committer
  identity *and* timestamps (via the `Clock` port) to keep SHAs stable across machines.
- Open: exact list of `testkit` builders may grow as P1–P6 discover fixture needs; treat
  the P0 set as a baseline, extend per phase.
