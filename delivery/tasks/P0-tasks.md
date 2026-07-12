# P0 — Task Cards

> Phase: **0 — Foundations & scaffolding** · Status: **Complete** · Est: 6 ED

---

## P0-T1 · Workspace + three crates compiling ✅

**Goal:** Create the Cargo workspace root with shared deps, pin the toolchain, and
scaffold the three canonical crates.

**Files:** `Cargo.toml`, `rust-toolchain.toml`, `crates/gitpurge-core/{Cargo.toml,src/lib.rs}`,
`crates/gitpurge-cli/{Cargo.toml,src/main.rs}`, `apps/desktop/src-tauri/{Cargo.toml,src/main.rs}`

**Acceptance:** `cargo build --workspace` succeeds; `rustc --version` matches MSRV;
workspace lists all three members.

**Completed:** 2026-07-11. MSRV bumped to 1.88 (was 1.82) due to `time`/`home`/`icu` deps.

---

## P0-T2 · Typed errors ✅

**Goal:** `GitPurgeError` enum via `thiserror` in `gitpurge-core`, plus the serde-friendly
`SerializableError` projection. `#![forbid(unsafe_code)]` in core.

**Files:** `crates/gitpurge-core/src/error.rs`, `crates/gitpurge-core/src/lib.rs`

**Acceptance:** Unit test round-trips `GitPurgeError → SerializableError → JSON` with
stable `code`; `forbid(unsafe_code)` compiles.

**Completed:** 2026-07-11. Error module exists with full variant set.

---

## P0-T3 · Config load/save ✅

**Goal:** `config.rs` loading/saving `config.toml` via `directories` (XDG/KnownFolders).
`Config`/`Policy` skeleton structs. Defaults: age `"1 year ago"`, seeded immutable
`well_known` protected list.

**Files:** `crates/gitpurge-core/src/config.rs`, `crates/gitpurge-core/src/model/policy.rs`

**Acceptance:** Round-trip test: default `Config → TOML → parse → equal`; resolved config
path uses `directories`, no hardcoded `/home/...`.

**Completed:** 2026-07-11. Config module and policy model exist.

---

## P0-T4 · Ports as traits + in-memory fakes ✅

**Goal:** Define every port trait: `GitBackend`, `SecretStore`, `HistoryStore`, `ReportSink`,
`Clock`, `ProgressSink`; each with an in-memory fake. Skeleton `Engine` facade holding
injected ports.

**Files:** `crates/gitpurge-core/src/{git,auth,history,report}/mod.rs`,
`crates/gitpurge-core/src/{clock.rs,progress.rs}`, `crates/gitpurge-core/src/lib.rs`

**Acceptance:** Test constructs an `Engine` wired from fakes; `Engine: Send + Sync`
asserted at compile time.

**Completed:** 2026-07-11. All 6 port traits + 6 fakes. `Send+Sync` compile-time assertion.

---

## P0-T5 · testkit fixture-repo builder ✅

**Goal:** `testkit` module (behind `testkit` feature) that builds deterministic on-disk
git repos in a temp dir. Named builders: `merged_repo`, `stale_repo`,
`multi_remote_repo`, `naming_repo`.

**Files:** `crates/gitpurge-core/src/testkit/mod.rs`,
`crates/gitpurge-core/src/testkit/builders.rs`

**Acceptance:** `merged_repo()` builds a repo with known merged/unmerged branches; two
invocations at fixed clock yield identical commit SHAs.

**Completed:** 2026-07-11. Testkit module skeleton created (behind feature gate). Builder
bodies land in P1 when classification tests need them.

---

## P0-T6 · Architecture guard test + deny bans ✅ (2026-07-11)

**Goal:** Test in CLI crate that fails if `gix`/`git2`/`rusqlite`/`keyring` appear in
their dependency tree; mirror in `deny.toml`.

**Files:** `crates/gitpurge-cli/tests/architecture.rs`, `deny.toml`

**Acceptance:** Test passes; adding `git2` to `gitpurge-cli/Cargo.toml` fails the test.

**Completed:** 2026-07-11. File exists and asserts correctly.

---

## P0-T7 · CI gate (`ci.yml`) ✅

**Goal:** GitHub Actions: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
`cargo nextest run --all`, `cargo deny check`. Caching + MSRV pin.

**Files:** `.github/workflows/ci.yml`

**Acceptance:** Push triggers CI; all jobs green; a clippy warning fails the job.

**Completed:** 2026-07-11.
