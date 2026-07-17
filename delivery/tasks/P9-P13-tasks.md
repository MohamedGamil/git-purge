# P9–P13 — Retrofit Task Cards

> Phases: **9–13 — Safety, Structure, Auth, Reporting, Desktop & Packaging Retrofit** · Status: **Draft** · Total Est: ~41.5 ED

---

## Phase 9 — Safety Retrofit & Testing Debt (Critical)

> Est: 8.25 ED · Priority: **Critical** · Depends: P0–P4

---

### P9-T1 · Named safety regression tests (SAFE-01–07) ✅ (2026-07-13)

**Goal:** Create 7 named tests, one per safety invariant. These tests may **never**
be removed (DoD §6).

**Files:** `crates/gitpurge-core/tests/safety.rs`

**Priority:** Quick Win (high impact, low effort)

**Acceptance:** All 7 tests pass; `cargo nextest run -E 'test(safe_0)'` runs all 7.

---

### P9-T2 · Core integration test directory ✅ (2026-07-17)

**Goal:** Create `crates/gitpurge-core/tests/` with round-trip tests: scan → plan →
backup → execute → restore.

**Files:** `crates/gitpurge-core/tests/integration.rs`

**Priority:** Quick Win

**Acceptance:** `cargo nextest run -p gitpurge-core --test integration` passes.

---

### P9-T3 · Desktop architecture guard test ✅ (2026-07-11)

**Goal:** Mirror CLI's `architecture.rs` for the desktop crate (ADR-0001 compliance).

**Files:** `apps/desktop/src-tauri/tests/architecture.rs`

**Priority:** Quick Win

**Acceptance:** Test passes; adding `gix` to desktop `Cargo.toml` fails it.

---

### P9-T4 · `insta` snapshot tests for CLI ✅ (2026-07-17)

**Goal:** Snapshot tests for every major CLI verb in human and JSON modes.

**Files:** `crates/gitpurge-cli/tests/cli_snapshots.rs`, `crates/gitpurge-cli/tests/snapshots/*`

**Priority:** High (high impact, high effort)

**Acceptance:** `cargo insta test -p gitpurge-cli` passes; snapshots committed.

---

### P9-T5 · Coverage gate in CI ✅ (2026-07-17)

**Goal:** Add `cargo llvm-cov` to CI with ≥50% gate, ramp plan to ≥80%.

**Files:** `.github/workflows/ci.yml`, `Makefile`

**Priority:** High

**Acceptance:** CI produces coverage report; `gitpurge-core` coverage ≥50%.

---

### P9-T6 · Remove `cmd/stubs.rs` or implement ✅ (2026-07-13)

**Goal:** Audit and eliminate `stubs.rs`. Implement real handlers or replace with
tracked `todo!("P<n>-T<m>")`.

**Files:** `crates/gitpurge-cli/src/cmd/stubs.rs`, `crates/gitpurge-cli/src/cmd/mod.rs`

**Priority:** Quick Win

**Acceptance:** No `stubs.rs` file exists.

---

### P9-T7 · Exit-code stability tests ✅ (2026-07-17)

**Goal:** Tests asserting stable exit codes. Document the exit-code map.

**Files:** `crates/gitpurge-cli/tests/exit_codes.rs`, `docs/05-cli-spec.md`

**Priority:** Medium (low impact, low effort)

**Acceptance:** Exit-code tests pass; codes documented.

---

### P9-T8 · Reconcile delivery task cards ✅ (2026-07-13)

**Goal:** Update all task cards to match codebase reality. Fix P0-T6 status, P5/P7
status, task-numbering divergences.

**Files:** `delivery/tasks/*.md`

**Priority:** Quick Win

**Acceptance:** Every task card status matches reality.

---

## Phase 10 — Structural Consolidation & Spec Alignment (High)

> Est: 6.25–7.25 ED · Priority: **High** · Depends: P9

---

### P10-T1 · Extract delete action into `action/delete.rs` ✅ (2026-07-17)

**Goal:** Move ~200 lines of delete orchestration from `lib.rs` into standalone module.

**Files:** `crates/gitpurge-core/src/action/delete.rs`, `crates/gitpurge-core/src/action/mod.rs`,
`crates/gitpurge-core/src/lib.rs`

**Priority:** Quick Win

**Acceptance:** `action/delete.rs` exists; all P9 safety tests pass.

---

### P10-T2 · Break up `lib.rs` into Engine modules ✅ 2026-07-17

**Goal:** Split `lib.rs` (65KB/1874 lines) into ≤500-line focused modules under `engine/`.

**Files:** `crates/gitpurge-core/src/engine/*.rs`, `crates/gitpurge-core/src/lib.rs`

**Priority:** High

**Acceptance:** `lib.rs` ≤200 lines (re-exports); each module ≤500 lines; all tests pass.

---

### P10-T3 · Split Tauri `commands.rs` into domain modules 🔲

**Goal:** Split 68KB `commands.rs` into per-domain modules under `commands/`.

**Files:** `apps/desktop/src-tauri/src/commands/*.rs`

**Priority:** Medium (low impact, low effort)

**Acceptance:** No single file exceeds 300 lines; desktop app builds and launches.

---

### P10-T4 · Resolve license inconsistency ✅ (2026-07-17)

**Goal:** Apache-2.0 only or dual-license — decide, then make everything consistent.

**Files:** `LICENSE*`, `Cargo.toml`, `delivery/CONVENTIONS.md`, `delivery/phase-7-packaging.md`

**Priority:** Quick Win

**Acceptance:** All license references agree; `cargo deny check licenses` passes.

---

### P10-T5 · Update CONVENTIONS and AGENTS.md for ADR-0006 ✅ (2026-07-17)

**Goal:** Replace `forbid(unsafe_code)` references with `deny(unsafe_code)` per ADR-0006.

**Files:** `delivery/CONVENTIONS.md`, `.agents/AGENTS.md`

**Priority:** Quick Win

**Acceptance:** No document references `forbid(unsafe_code)` for `gitpurge-core`.

---

### P10-T6 · Amend ADR-0002 or build `ShellGitBackend` ✅ (2026-07-17)

**Goal:** Either build the shell-out adapter or formally amend the ADR to drop it.

**Files:** Option (a): `crates/gitpurge-core/src/git/shell_backend.rs` ·
Option (b): `docs/adr/ADR-0002-git-engine-hybrid.md`

**Priority:** High (if build) / Quick Win (if amend)

**Acceptance:** Shell backend passes `GitBackend` trait tests, or ADR status is "Amended."

---

### P10-T7 · Expand CI to cross-platform matrix 🔲

**Goal:** CI runs on Linux + macOS + Windows.

**Files:** `.github/workflows/ci.yml`

**Priority:** Medium

**Acceptance:** `cargo nextest run` passes on all 3 platforms.

---

## Phase 11 — Authentication & Credential Management (Critical)

> Est: 8.5 ED · Priority: **Critical** · Depends: P9, P10

---

### P11-T1 · Credential model types ✅ (2026-07-17)

**Goal:** Full credential type hierarchy with redacted `Debug` (SAFE-07).

**Files:** `crates/gitpurge-core/src/auth/mod.rs`, `crates/gitpurge-core/src/auth/credential.rs`

**Priority:** Quick Win

**Acceptance:** `Debug` output never contains token strings.

---

### P11-T2 · Keyring `SecretStore` adapter ✅ (2026-07-17)

**Goal:** `SecretStore` backed by OS keychain via `keyring` crate.

**Files:** `crates/gitpurge-core/src/auth/keyring_store.rs`

**Priority:** High

**Acceptance:** Store → retrieve → delete cycle passes on a system with a keychain.

---

### P11-T3 · Encrypted file `SecretStore` fallback ✅ (2026-07-17)

**Goal:** `SecretStore` backed by AES-256-GCM encrypted JSON file.

**Files:** `crates/gitpurge-core/src/auth/file_store.rs`

**Priority:** High

**Acceptance:** File on disk is not valid JSON; wrong passphrase fails gracefully.

---

### P11-T4 · Credential resolver chain ✅ (2026-07-17)

**Goal:** Resolver tries sources in order: CLI flag → keyring → file → env → prompt.

**Files:** `crates/gitpurge-core/src/auth/resolver.rs`

**Priority:** High

**Acceptance:** Resolver returns first successful credential in priority order.

---

### P11-T5 · git2 credential callback bridge ✅ (2026-07-17)

**Goal:** Wire resolver into git2's `RemoteCallbacks::credentials()`.

**Files:** `crates/gitpurge-core/src/git/git2_backend.rs`

**Priority:** High

**Acceptance:** `fetch` on authenticated remotes works via the credential bridge.

---

### P11-T6 · `safe_07` secret hygiene regression suite ✅ (2026-07-17)

**Goal:** Comprehensive test asserting no credential material in any output path.

**Files:** `crates/gitpurge-core/tests/safe_07_secrets.rs`

**Priority:** Quick Win

**Acceptance:** Injected token string not found in any output/error/log.

---

### P11-T7 · Wire auth CLI commands to real backends ✅ 2026-07-17

**Goal:** Replace auth command stubs with real `Engine::auth_*` calls.

**Files:** `crates/gitpurge-cli/src/cmd/auth.rs`

**Priority:** Medium

**Acceptance:** `git-purge auth add/list/remove/test` all functional.

---

## Phase 12 — Reporting Completion & Hardening Foundations (High)

> Est: 10.5 ED · Priority: **High** · Depends: P9, P10

---

### P12-T1 · Implement trend diffs 🔲

**Goal:** Replace 782B stub with full trend-diff implementation.

**Files:** `crates/gitpurge-core/src/history/trends.rs`, Engine wiring

**Priority:** Quick Win

**Acceptance:** Two `ScanResult`s → `TrendDiff` with correct deltas.

---

### P12-T2 · `insta` golden tests for report renderers 🔲

**Goal:** Golden-file tests for markdown, HTML, JSON report formats.

**Files:** `crates/gitpurge-core/tests/report_golden.rs`, `crates/gitpurge-core/tests/snapshots/*`

**Priority:** Quick Win

**Acceptance:** `cargo insta test` passes; snapshots committed for all 3 formats.

---

### P12-T3 · `history import --legacy-json` 🔲

**Goal:** CLI command to import legacy `track_branch_trends.py` JSON data.

**Files:** `crates/gitpurge-cli/src/cmd/reporting.rs`, `crates/gitpurge-core/src/history/sqlite.rs`

**Priority:** Medium (low impact, low effort)

**Acceptance:** Import succeeds; `history` shows imported data.

---

### P12-T4 · `proptest` property tests for policy engine 🔲

**Goal:** Property tests for age parsing, naming classification, protection resolution.

**Files:** `crates/gitpurge-core/src/policy/*.rs`, `Cargo.toml`

**Priority:** High

**Acceptance:** No panics on arbitrary inputs; each policy module has ≥1 property test.

---

### P12-T5 · Performance benchmark suite 🔲

**Goal:** `criterion` benchmarks for scan/plan/report on fixture repos.

**Files:** `crates/gitpurge-core/benches/engine_bench.rs`, `Makefile`

**Priority:** Medium

**Acceptance:** `cargo bench` runs and produces timing results.

---

### P12-T6 · Frontend Vitest expansion 🔲

**Goal:** Unit tests for the 6 untested Vue views.

**Files:** `apps/desktop/src/views/*.spec.ts`

**Priority:** Medium

**Acceptance:** `pnpm test` passes; each view has a `.spec.ts`; views coverage ≥60%.

---

### P12-T7 · `SECURITY.md` threat-model content 🔲

**Goal:** Expand skeletal `SECURITY.md` with threat-model from `14-security.md`.

**Files:** `SECURITY.md`

**Priority:** Medium

**Acceptance:** Covers all threat categories; references code locations for mitigations.

---

## Phase 13 — Desktop Polish & Packaging Finalization (Medium)

> Est: 8 ED · Priority: **Medium** · Depends: P9, P10, P11

---

### P13-T1 · Pinia stores for remaining views 🔲

**Goal:** Pinia stores for branches, backups, history, settings with tests.

**Files:** `apps/desktop/src/stores/*.ts`, `apps/desktop/src/stores/*.spec.ts`

**Priority:** Quick Win

**Acceptance:** Each store has CRUD ops + `.spec.ts`; views use stores.

---

### P13-T2 · `tauri-driver` + WebDriver e2e smoke suite 🔲

**Goal:** Headless e2e tests: launch → scan → plan → theme switch. ≥5 scenarios.

**Files:** `apps/desktop/tests/e2e/*.spec.ts`

**Priority:** High

**Acceptance:** `pnpm e2e` runs headless; 5+ scenarios pass.

---

### P13-T3 · Accessibility pass 🔲

**Goal:** Keyboard nav, ARIA labels, focus indicators, WCAG AA contrast.

**Files:** `apps/desktop/src/views/*.vue`, `apps/desktop/src/styles/*.css`

**Priority:** Medium

**Acceptance:** All interactive elements reachable by Tab; no contrast below 4.5:1.

---

### P13-T4 · SHA256 checksums + signing verification 🔲

**Goal:** Automate checksums + minisign signatures in `release.yml`.

**Files:** `.github/workflows/release.yml`, `scripts/verify-release.sh`

**Priority:** Quick Win

**Acceptance:** Release produces `SHA256SUMS` + `.sig`; verification script works.

---

### P13-T5 · Cross-platform release verification 🔲

**Goal:** Post-release CI job runs `git-purge --version` from downloaded tarball on
all 3 platforms.

**Files:** `.github/workflows/release.yml`

**Priority:** Medium

**Acceptance:** Binary executes on all platforms; version matches tag.

---

### P13-T6 · `git-purge show` as separate CLI command 🔲

**Goal:** Standalone `show <ref> [<path>]` command per CONVENTIONS §9.

**Files:** `crates/gitpurge-cli/src/cmd/show.rs`, `crates/gitpurge-cli/src/cli.rs`

**Priority:** Low (low impact, high effort relative to value)

**Acceptance:** `git-purge show main` lists tree; `show main:README.md` prints content.
