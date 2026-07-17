# Phase 9 — Safety Retrofit & Testing Debt (Critical)

`Status: Complete` · `Owner: Delivery` · `Last-updated: 2026-07-17` ·
`Related: [DEFINITION_OF_DONE](DEFINITION_OF_DONE.md), [11-safety-model.md](../docs/11-safety-model.md), [12-testing-strategy.md](../docs/12-testing-strategy.md), [analysis](../.scratch/analysis/delivery-vs-specs-analysis.md)`

## Goal

Retrofit the **critical testing gaps** identified in the project analysis. The DoD
requires named safety regression tests (SAFE-01–07), integration tests in
`crates/*/tests/`, `insta` CLI snapshot tests, and coverage measurement — none of
which exist today. This phase is a prerequisite for any further feature work because
every subsequent change will lack a safety net without it.

Tasks are ordered by: (1) critical path dependencies, (2) quick wins first, then
high-impact items, following the priority matrix:
**Quick Win** = high impact, low effort · **High** = high impact, high effort ·
**Medium** = low impact, low effort · **Low** = low impact, high effort.

**Milestone:** M-Retrofit-1 — Safety & testing baseline established.

**Dependencies:** P0–P4 completed (existing code).

## Tasks

| Task ID | Title | Description | Files (repo-relative) | Depends-on | ∥? | Est (ED) | Priority | Acceptance test |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| **P9-T1** | Named safety regression tests (SAFE-01–07) | Create a dedicated test module with 7 named tests, one per safety invariant: `safe_01_dry_run_default`, `safe_02_protected_refs_never_deleted`, `safe_03_tags_never_deleted_by_branch_ops`, `safe_04_verified_pre_op_snapshot`, `safe_05_failed_delete_offers_restore`, `safe_06_no_force_overwrite_restore`, `safe_07_no_secrets_in_output`. Each uses testkit fixtures and asserts the invariant end to end. These tests may **never** be removed (DoD §6). | `crates/gitpurge-core/tests/safety.rs` | — | no | 2 | Quick Win | All 7 tests pass; each test is named exactly per convention; `cargo nextest run -E 'test(safe_0)' ` runs all 7; removing any one fails DoD review. |
| **P9-T2** | Core integration test directory | Create `crates/gitpurge-core/tests/` with round-trip integration tests: scan → plan → backup → execute → restore. Tests use testkit fixtures, exercise the Engine through its public API, and cover the scan→delete→verify-backup→restore flow. | `crates/gitpurge-core/tests/integration.rs` | P9-T1 | no | 2 | Quick Win | `cargo nextest run -p gitpurge-core --test integration` passes; covers at least: scan finding merged branches, plan producing correct items, execute deleting and recording, restore recovering the deleted branch. |
| **P9-T3** | Desktop architecture guard test | Mirror the CLI's `architecture.rs` for the desktop crate. Verify that `gitpurge-desktop` does not directly depend on `gix`/`git2`/`rusqlite`/`keyring` in its `Cargo.toml`. Closes ADR-0001 partial compliance. | `apps/desktop/src-tauri/tests/architecture.rs` | — | yes | 0.25 | Quick Win | Test passes; adding `gix` to `gitpurge-desktop/Cargo.toml` fails the test. |
| **P9-T4** | `insta` snapshot tests for CLI | Add `insta` as a dev-dependency to `gitpurge-cli`. Create snapshot tests for every major CLI verb: `scan`, `plan`, `delete --dry-run`, `backup list`, `diff`, `report`. Cover both human table output and `--json` output. Commit snapshot files. | `crates/gitpurge-cli/tests/cli_snapshots.rs`, `crates/gitpurge-cli/tests/snapshots/*` | P9-T1 | yes | 2 | High | `cargo insta test -p gitpurge-cli` passes; snapshot files committed; changing output format causes snapshot mismatch (caught by CI). |
| **P9-T5** | Coverage gate in CI | Add `cargo llvm-cov` (or `cargo tarpaulin`) step to `ci.yml`. Set initial gate at ≥50% for `gitpurge-core`, with documented plan to ramp to ≥80%. Upload coverage report as artifact. | `.github/workflows/ci.yml`, `Makefile` (coverage target) | P9-T1, P9-T2 | yes | 0.5 | High | CI job produces coverage report; `gitpurge-core` coverage ≥50%; job fails if coverage drops below gate. |
| **P9-T6** | Remove `cmd/stubs.rs` or implement | Audit `crates/gitpurge-cli/src/cmd/stubs.rs` (833B). For each stub: either implement the real command handler (if trivial) or replace with explicit `todo!("P<n>-T<m>")` tracking the delivery task that will fill it. Delete the stubs file when done. | `crates/gitpurge-cli/src/cmd/stubs.rs`, `crates/gitpurge-cli/src/cmd/mod.rs` | — | yes | 0.5 | Quick Win | No `stubs.rs` file exists; any remaining placeholder commands use `todo!()` with a task ID reference. |
| **P9-T7** | Exit-code stability tests | Add tests asserting stable exit codes for success (0), user refusal (1), error (2), config error (78). Document the exit-code map in `docs/05-cli-spec.md`. | `crates/gitpurge-cli/tests/exit_codes.rs`, `docs/05-cli-spec.md` | P9-T4 | yes | 0.5 | Medium | Exit-code tests pass; each code is documented; changing an exit code breaks the test. |
| **P9-T8** | Reconcile delivery task cards | Update `delivery/tasks/P0-tasks.md` through `P5-P8-tasks.md` to reflect actual codebase state: mark P0-T6 as complete (file exists), update P5 items that are implemented (SQLite store, report renderers), update P7 items (`release.yml`, tarball). Fix task-numbering divergences between phase docs and task cards. | `delivery/tasks/*.md` | — | yes | 0.5 | Quick Win | Every task card status matches codebase reality; no "Not started" for implemented work; task IDs are consistent between phase docs and task cards. |

Total ≈ 8.25 ED.

## Exit criteria

- All 7 named safety regression tests (`safe_01`–`safe_07`) pass in CI and are
  committed — they may never be removed (DoD §6).
- `crates/gitpurge-core/tests/` directory exists with passing integration tests.
- `insta` snapshot tests cover every major CLI verb in human and JSON modes.
- CI reports code coverage with a ≥50% gate on `gitpurge-core`.
- Architecture guard test exists in both CLI and desktop crates.
- Delivery task cards accurately reflect codebase reality.
- `cmd/stubs.rs` is eliminated.

### Requirements & safety invariants satisfied

- **R8** (testing mandatory): P9-T1, P9-T2, P9-T4, P9-T5, P9-T7.
- **R6** (shared abstractions guarded structurally): P9-T3.
- **SAFE-01** through **SAFE-07**: each has a dedicated, named regression test (P9-T1).

## Risks & open questions

- **Testkit maturity.** P9-T1 and P9-T2 depend on testkit builders being rich enough
  to produce fixture repos for every safety scenario. May need to extend testkit as
  part of this phase.
- **Coverage gate level.** Starting at ≥50% is pragmatic given 30 existing tests;
  ramp schedule to ≥80% should be documented and reviewed at each phase boundary.
- **`insta` snapshot churn.** Output format changes in P5/P10 will break snapshots;
  that's intentional (catches accidental regressions) but requires awareness.

---

# Phase 10 — Structural Consolidation & Spec Alignment (High)

`Status: Complete` · `Owner: Delivery` · `Last-updated: 2026-07-17` ·
`Related: [02-architecture.md](../docs/02-architecture.md), [ADR-0001](../docs/adr/ADR-0001-hexagonal-architecture.md), [ADR-0002](../docs/adr/ADR-0002-git-engine-hybrid.md), [ADR-0006](../docs/adr/ADR-0006-unsafe-libgit2-timeouts.md), [CONVENTIONS](CONVENTIONS.md), [analysis](../.scratch/analysis/delivery-vs-specs-analysis.md)`

## Goal

Resolve **structural deviations** from the spec and ADR decisions: break up the
65KB `lib.rs` monolith, extract delete logic into its spec'd module, split the 68KB
Tauri `commands.rs`, reconcile the license question, and update documentation that
has drifted from reality. This phase makes the codebase match what the architecture
documents describe.

**Milestone:** M-Retrofit-2 — Codebase structure matches spec.

**Dependencies:** **P9** (tests exist to catch regressions during refactoring).

## Tasks

| Task ID | Title | Description | Files (repo-relative) | Depends-on | ∥? | Est (ED) | Priority | Acceptance test |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| **P10-T1** | Extract delete action into `action/delete.rs` | Move the ~200 lines of delete orchestration from `lib.rs` `execute_with_progress` into a dedicated `action/delete.rs` module per the Phase 3 spec. The Engine method calls the new module. All existing tests still pass. | `crates/gitpurge-core/src/action/delete.rs`, `crates/gitpurge-core/src/action/mod.rs`, `crates/gitpurge-core/src/lib.rs` | P9-T1 | no | 0.5 | Quick Win | `action/delete.rs` exists with the extracted logic; `lib.rs` calls it; all P9 safety tests pass; `cargo nextest run` green. |
| **P10-T2** | Break up `lib.rs` into Engine modules | Split `lib.rs` (65KB / 1874 lines) into focused modules: `engine/mod.rs` (struct + constructor + config), `engine/repo.rs` (repo CRUD), `engine/scan.rs` (scan + plan), `engine/backup.rs` (backup ops), `engine/action.rs` (execute + restore + archive), `engine/report.rs` (report + history + diff). `lib.rs` re-exports the public API. | `crates/gitpurge-core/src/engine/*.rs`, `crates/gitpurge-core/src/lib.rs` | P10-T1 | no | 2.5 | High | `lib.rs` is ≤200 lines (re-exports only); each engine module is ≤500 lines; public API unchanged; all tests pass; `cargo doc` generates clean docs. |
| **P10-T3** | Split Tauri `commands.rs` into domain modules | Split `commands.rs` (68KB) into per-domain modules: `commands/mod.rs` (re-exports), `commands/types.rs` (32 DTO structs), `commands/mappers.rs` (18 mapping functions), `commands/repo.rs`, `commands/scan.rs`, `commands/backup.rs`, `commands/delete.rs`, `commands/diff.rs`, `commands/report.rs`, `commands/settings.rs`, `commands/auth.rs`. | `apps/desktop/src-tauri/src/commands/*.rs` | — | yes | 1 | Medium | No single file exceeds 300 lines; all 31 Tauri commands still register and respond; desktop app builds and launches. |
| **P10-T4** | Resolve license inconsistency | Decide: Apache-2.0 only (per CONVENTIONS §1) or dual Apache-2.0 + MIT (per codebase). Update `LICENSE*` files, `Cargo.toml` `license` field, `deny.toml` allow-list, CONVENTIONS §1, and phase-7 doc to be consistent. | `LICENSE`, `LICENSE-APACHE`, `LICENSE-MIT` (conditionally), `Cargo.toml`, `delivery/CONVENTIONS.md`, `delivery/phase-7-packaging.md` | — | yes | 0.25 | Quick Win | All license references agree; `cargo deny check licenses` passes; no contradictions between CONVENTIONS and codebase. |
| **P10-T5** | Update CONVENTIONS and AGENTS.md for ADR-0006 | CONVENTIONS §11 and `.agents/AGENTS.md` still reference `#![forbid(unsafe_code)]` for `gitpurge-core`. Update both to document that core uses `#![deny(unsafe_code)]` per ADR-0006, with a single `#[allow(unsafe_code)]` exemption for libgit2 global timeout configuration. | `delivery/CONVENTIONS.md`, `.agents/AGENTS.md` | — | yes | 0.25 | Quick Win | No document references `forbid(unsafe_code)` for `gitpurge-core`; ADR-0006 is cited as the authoritative decision. |
| **P10-T6** | Amend ADR-0002 or build `ShellGitBackend` | ADR-0002 §4 specifies a `ShellGitBackend` as a last-resort adapter. Either: (a) implement `ShellGitBackend` with `git credential fill` + basic ref operations and proper command-escaping (SAFE-07), or (b) amend ADR-0002 status to "Amended" and remove the shell-out clause, documenting why it was dropped. | Option (a): `crates/gitpurge-core/src/git/shell_backend.rs` · Option (b): `docs/adr/ADR-0002-git-engine-hybrid.md` | — | yes | 1.5 (build) or 0.25 (amend) | High (if build) / Quick Win (if amend) | If built: `ShellGitBackend` passes the same `GitBackend` trait tests as gix/git2; command strings are sanitized. If amended: ADR-0002 status is "Amended," rationale documented, no code references the shell backend. |
| **P10-T7** | Expand CI to cross-platform matrix | Expand `ci.yml` from single-platform (ubuntu-latest) to a matrix of Linux + macOS + Windows. Add Tauri system dep installation for desktop builds where feasible. Frontend (`pnpm test`) gate added. | `.github/workflows/ci.yml` | — | yes | 1 | Medium | CI matrix runs on all 3 platforms; at least `cargo nextest run --workspace --exclude gitpurge-desktop` passes on all; frontend tests run on Linux. |

Total ≈ 6.25–7.25 ED (depending on P10-T6 decision).

## Exit criteria

- `lib.rs` is ≤200 lines (re-exports only); Engine logic is in focused modules.
- `action/delete.rs` exists as a standalone module.
- Tauri `commands.rs` is split into ≤300-line domain modules.
- All license references are consistent across the project.
- CONVENTIONS and AGENTS.md reflect the `deny(unsafe_code)` reality.
- ADR-0002 is either fully implemented (shell backend) or formally amended.
- CI runs on at least 2 platforms.

### Requirements & safety invariants satisfied

- **R6** (shared abstractions, thin adapters): P10-T2, P10-T3 restore the "thin
  adapter" intent of ADR-0001.
- **R8** (testing mandatory): refactoring under P9's test coverage net.
- **R11** (cross-platform CI): P10-T7.

## Risks & open questions

- **Refactoring risk.** P10-T2 is a large structural refactor. P9's test coverage
  is the safety net — if coverage is too thin, defer P10-T2 until more tests exist.
- **Shell backend decision.** P10-T6 is a design fork: building the shell backend
  adds 1.5 ED and a maintenance surface; amending the ADR is 0.25 ED but sets a
  precedent for ADR drift. Recommend deciding before session starts.
- **Cross-platform CI cost.** macOS runners are 10× more expensive than Linux on
  GitHub Actions. Consider running macOS only on `main` merges, not PRs.

---

# Phase 11 — Authentication & Credential Management (Critical)

`Status: Complete` · `Owner: Delivery` · `Last-updated: 2026-07-17` ·
`Related: [09-authentication.md](../docs/09-authentication.md), [14-security.md](../docs/14-security.md), [ADR-0002](../docs/adr/ADR-0002-git-engine-hybrid.md), [phase-6-auth.md](phase-6-auth.md), [analysis](../.scratch/analysis/delivery-vs-specs-analysis.md)`

## Goal

Implement the **authentication and credential management** system that was spec'd
in Phase 6 but never built beyond a trait definition and CLI stub. Without this,
Git Purge cannot interact with any authenticated remote — the primary real-world
use case. This is the single largest functional gap blocking production use.

**Milestone:** M-Retrofit-3 — Authenticated remote operations functional.

**Dependencies:** **P9** (safety tests), **P10** (structural consolidation, especially
P10-T1 so delete action is cleanly separated).

## Tasks

| Task ID | Title | Description | Files (repo-relative) | Depends-on | ∥? | Est (ED) | Priority | Acceptance test |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| **P11-T1** | Credential model types | Define the full credential type hierarchy: `CredentialKind` (SSH key, HTTPS token, HTTPS username/password), `Credential` struct with redacted `Debug` (SAFE-07), `CredentialQuery` for resolution lookups. Extend the existing `auth/mod.rs` trait. | `crates/gitpurge-core/src/auth/mod.rs`, `crates/gitpurge-core/src/auth/credential.rs` | — | no | 0.5 | Quick Win | Unit test: `Credential` with a token value has a `Debug` output that never contains the token string; `CredentialKind` enum covers SSH/HTTPS-token/HTTPS-basic. |
| **P11-T2** | Keyring `SecretStore` adapter | Implement `SecretStore` trait backed by the `keyring` crate (OS keychain: macOS Keychain, Windows Credential Manager, Linux Secret Service/kwallet). Store/retrieve/delete credentials keyed by `(repo_id, remote)`. | `crates/gitpurge-core/src/auth/keyring_store.rs` | P11-T1 | no | 1.5 | High | On a system with a keychain: store → retrieve → delete cycle passes; retrieved credential matches stored; no plaintext in logs (SAFE-07). |
| **P11-T3** | Encrypted file `SecretStore` fallback | Implement `SecretStore` backed by an encrypted JSON file (AES-256-GCM via `ring` or `aes-gcm`, key derived from machine-id + user passphrase via `argon2`). Fallback for headless servers / CI without a keychain. File stored at `$DATA_DIR/credentials.enc`. | `crates/gitpurge-core/src/auth/file_store.rs` | P11-T1 | yes | 2 | High | Round-trip: store credential → read back → matches; file on disk is not valid JSON (encrypted); wrong passphrase fails gracefully; no plaintext in error messages (SAFE-07). |
| **P11-T4** | Credential resolver chain | Implement a resolver that tries credential sources in order: (1) explicit CLI flag, (2) keyring store, (3) encrypted file store, (4) `GIT_ASKPASS` / `SSH_AUTH_SOCK` environment, (5) interactive prompt (if TTY). Returns the first successful credential or a clear error. | `crates/gitpurge-core/src/auth/resolver.rs` | P11-T2, P11-T3 | no | 1 | High | Resolver with a populated keyring returns the keyring credential; with empty keyring + populated file store, returns the file credential; with nothing, falls through to error (or prompt in TTY). |
| **P11-T5** | git2 credential callback bridge | Wire the credential resolver into git2's `RemoteCallbacks::credentials()` callback so that `push --delete` and `fetch` on authenticated remotes work transparently. Handle SSH agent, HTTPS token, and username/password flows. | `crates/gitpurge-core/src/git/git2_backend.rs` | P11-T4 | no | 1.5 | High | Integration test: fetch from a local file:// remote works; credential callback is invoked for HTTPS remotes (mock server or env-based test); SSH agent forwarding is attempted when `SSH_AUTH_SOCK` is set. |
| **P11-T6** | `safe_07` secret hygiene regression suite | Create a comprehensive test that exercises error paths, log output, scan results, report rendering, and snapshot metadata — asserting that no credential material appears anywhere. This fills the SAFE-07 gap identified in the analysis. | `crates/gitpurge-core/tests/safe_07_secrets.rs` | P11-T1 | yes | 1 | Quick Win | Test injects a credential with a known token string; exercises every code path that touches credentials; greps all output/errors/logs for the token string; asserts zero matches. |
| **P11-T7** | Wire auth CLI commands to real backends | Replace the stub `cmd/auth.rs` handlers with real calls to `Engine::auth_store`, `auth_retrieve`, `auth_remove`, `auth_list`, `auth_test`. Add `--backend keyring|file` flag. | `crates/gitpurge-cli/src/cmd/auth.rs` | P11-T4 | yes | 1 | Medium | `git-purge auth add --backend keyring` stores a credential; `auth list` shows it (redacted); `auth test` validates it against a remote; `auth remove` deletes it. |

Total ≈ 8.5 ED.

## Exit criteria

- `git-purge delete --execute` works on an authenticated HTTPS remote.
- Credential storage works via OS keychain (primary) and encrypted file (fallback).
- Credential resolver tries sources in documented priority order.
- SAFE-07 regression suite passes: no secret material in any output path.
- `auth` CLI commands are fully functional (not stubs).

### Requirements & safety invariants satisfied

- **R5** (multiple auth methods + secure storage): P11-T1 through P11-T5.
- **SAFE-07** (no secrets in logs/errors/snapshots/reports): P11-T1 (redacted Debug),
  P11-T6 (regression suite).

## Risks & open questions

- **Keychain availability.** Headless CI and Docker containers lack a keychain. Tests
  must detect this and skip keyring tests gracefully (with a clear "skipped: no
  keychain available" message), falling through to the file store.
- **Passphrase UX for encrypted file.** How does the user provide the passphrase?
  Options: env var `GITPURGE_PASSPHRASE`, interactive prompt, or derive from
  machine-id only (weaker). Recommend env var + prompt fallback.
- **SSH agent.** SSH_AUTH_SOCK forwarding is platform-dependent. Test on Linux/macOS;
  document Windows limitations (pageant vs OpenSSH agent).

---

# Phase 12 — Reporting Completion & Hardening Foundations (High)

`Status: In Progress` · `Owner: Delivery` · `Last-updated: 2026-07-17` ·
`Related: [10-reporting-and-history.md](../docs/10-reporting-and-history.md), [12-testing-strategy.md](../docs/12-testing-strategy.md), [phase-5-reporting.md](phase-5-reporting.md), [phase-8-hardening-release.md](phase-8-hardening-release.md), [analysis](../.scratch/analysis/delivery-vs-specs-analysis.md)`

## Goal

Complete the **reporting and history** features that were partially implemented ahead
of schedule (SQLite store, 3 report renderers) but left with critical gaps (trend
diffs are a stub, no golden tests). Also establish the **hardening foundations**
(property tests, performance benchmarks) that Phase 8 requires but were entirely
unstarted.

**Milestone:** M-Retrofit-4 — Reporting complete, hardening foundations laid.

**Dependencies:** **P9** (test infrastructure), **P10** (structural consolidation).

## Tasks

| Task ID | Title | Description | Files (repo-relative) | Depends-on | ∥? | Est (ED) | Priority | Acceptance test |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| **P12-T1** | Implement trend diffs | Port `track_branch_trends.py` logic: compare two scan snapshots over time, compute branch-count deltas, identify newly appeared/disappeared branches, track merge velocity. Replace the 782B stub in `trends.rs` with a full implementation. Wire `Engine::trends()` method. | `crates/gitpurge-core/src/history/trends.rs`, `crates/gitpurge-core/src/lib.rs` (or `engine/report.rs`) | — | no | 1.5 | Quick Win | Unit test: two `ScanResult`s at different times → `TrendDiff` with correct added/removed/delta counts; wired through Engine. |
| **P12-T2** | `insta` golden tests for report renderers | Add `insta` golden-file tests for all 3 report formats (markdown, HTML, JSON). Each test renders a known `RunReport` fixture and snapshots the output. Catches accidental format regressions. | `crates/gitpurge-core/tests/report_golden.rs`, `crates/gitpurge-core/tests/snapshots/*` | — | yes | 1 | Quick Win | `cargo insta test -p gitpurge-core --test report_golden` passes; snapshot files committed for all 3 formats; changing a renderer breaks the snapshot. |
| **P12-T3** | `history import --legacy-json` | CLI command to import legacy `track_branch_trends.py` JSON data into the SQLite history store. Provides migration path for existing users. | `crates/gitpurge-cli/src/cmd/reporting.rs`, `crates/gitpurge-core/src/history/sqlite.rs` | P12-T1 | no | 1 | Medium | `git-purge history import legacy-data.json` imports records; `git-purge history` shows the imported data; invalid JSON produces a clear error. |
| **P12-T4** | `proptest` property tests for policy engine | Add `proptest` as a dev-dependency. Write property tests for: age threshold parsing (arbitrary durations), naming classification (arbitrary branch name strings), protection resolution (arbitrary glob patterns vs branch names). These catch edge cases that unit tests miss. | `crates/gitpurge-core/src/policy/age.rs`, `crates/gitpurge-core/src/policy/naming.rs`, `crates/gitpurge-core/src/policy/protection.rs`, `crates/gitpurge-core/Cargo.toml` | — | yes | 1.5 | High | `cargo nextest run -p gitpurge-core -E 'test(proptest)'` passes; each policy module has at least one property test; no panics on arbitrary inputs. |
| **P12-T5** | Performance benchmark suite | Create `benches/` directory with `criterion` benchmarks: scan a 500-branch fixture repo, generate a plan, render a markdown report. Establish baseline numbers. Add a `make bench` target. | `crates/gitpurge-core/benches/engine_bench.rs`, `Makefile` | — | yes | 1.5 | Medium | `cargo bench -p gitpurge-core` runs without error; produces timing results for scan/plan/report; results are saved for regression comparison. |
| **P12-T6** | Frontend Vitest expansion | Add Vitest unit tests for the remaining 6 untested Vue views: `AuthView`, `BackupsView`, `BranchesView`, `CleanupView`, `DiffView`, `HistoryView`. Each test: mounts the component with mock Tauri invoke, verifies key elements render, simulates basic user interaction. | `apps/desktop/src/views/*.spec.ts` | — | yes | 3 | Medium | `pnpm test` passes; each view has a `.spec.ts` file; coverage ≥60% for the `views/` directory. |
| **P12-T7** | `SECURITY.md` threat-model content | Expand the existing skeletal `SECURITY.md` with the threat-model checklist from `docs/14-security.md`: attack surfaces, mitigations, credential handling, command injection prevention, supply-chain security. Document the SAFE-01–07 invariants and their test coverage. | `SECURITY.md` | P11-T6 | yes | 1 | Medium | `SECURITY.md` covers all threat categories from `14-security.md`; references specific code locations for each mitigation; documents responsible disclosure process. |
| **P12-T8** | Multi-threaded branch delete/archive with progress reporting | Implement thread-local `git2::Repository` instances across multiple threads to run delete and archive tasks concurrently. Utilize a channel sender (e.g. crossbeam or Tokio threadpool) to stream progress updates back to the caller for event propagation to the frontend. | `crates/gitpurge-core/src/action/delete.rs`, `crates/gitpurge-core/src/engine/action.rs` | P10-T1, P10-T2 | no | 1.5 | High | Regression integration test deletes 10 local/remote branches concurrently and asserts all are deleted; progress counts are emitted sequentially (`1/10`, `2/10`, etc.) and captured by a mock listener. |

Total ≈ 12.0 ED.

## Exit criteria

- Trend diffs are fully implemented (not a stub) and wired through the Engine.
- All 3 report formats have `insta` golden tests.
- Property tests exist for the policy engine with no panics on arbitrary input.
- Performance benchmarks establish baseline numbers for scan/plan/report.
- Frontend test coverage ≥60% for the views directory.
- `SECURITY.md` contains substantive threat-model content.

### Requirements & safety invariants satisfied

- **R7** (reports + historical tracking integral): P12-T1, P12-T2, P12-T3.
- **R8** (testing mandatory): P12-T2, P12-T4, P12-T5, P12-T6.
- **SAFE-02** property-tested: P12-T4 (protection resolution with arbitrary globs).

## Risks & open questions

- **`proptest` + `gix` interaction.** Property tests may hit slow paths in gix if
  they create actual git repos. Consider using in-memory fakes for property tests
  and reserving gix for integration tests.
- **Benchmark stability.** `criterion` benchmarks on CI runners are noisy due to
  shared resources. Consider running benchmarks only locally or on dedicated runners.
- **Legacy JSON format.** Need to document/discover the exact schema of
  `track_branch_trends.py` output to implement P12-T3.

---

# Phase 13 — Desktop Polish & Packaging Finalization (Medium)

`Status: In Progress` · `Owner: Delivery` · `Last-updated: 2026-07-17` ·
`Related: [06-ui-spec.md](../docs/06-ui-spec.md), [07-ui-design-system.md](../docs/07-ui-design-system.md), [13-distribution-and-ci.md](../docs/13-distribution-and-ci.md), [ADR-0003](../docs/adr/ADR-0003-ui-vue-tauri.md), [phase-4-ui.md](phase-4-ui.md), [phase-7-packaging.md](phase-7-packaging.md), [analysis](../.scratch/analysis/delivery-vs-specs-analysis.md)`

## Goal

Close the remaining **desktop UI gaps** (e2e tests, Pinia store adoption,
accessibility) and finalize **packaging and distribution** (signing, checksums,
cross-platform release verification). These are the medium-priority items that
complete P4 and P7's unmet acceptance criteria.

**Milestone:** M-Retrofit-5 — Desktop and packaging production-ready.

**Dependencies:** **P9** (test infra), **P10** (commands.rs split), **P11** (auth
UI functional).

## Tasks

| Task ID | Title | Description | Files (repo-relative) | Depends-on | ∥? | Est (ED) | Priority | Acceptance test |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| **P13-T1** | Pinia stores for remaining views | Create Pinia stores for branches, backups, history, and settings state. Migrate local reactive state from views into stores. Each store has a companion `.spec.ts` test file. | `apps/desktop/src/stores/{branches,backups,history,settings}.ts`, `apps/desktop/src/stores/*.spec.ts` | — | no | 2 | Quick Win | Each store has CRUD operations; `.spec.ts` tests pass; views import from stores instead of managing local state. |
| **P13-T2** | `tauri-driver` + WebDriver e2e smoke suite | Set up `tauri-driver` with WebDriver (or Playwright) for headless e2e tests: launch app → navigate to Dashboard → scan a fixture repo → preview plan → switch theme → verify dark/light toggle. Minimum 5 e2e scenarios. | `apps/desktop/tests/e2e/*.spec.ts`, `apps/desktop/tauri-driver.config.ts` | P13-T1 | no | 2 | High | `pnpm e2e` runs headless; 5+ scenarios pass; theme switch produces visible CSS variable changes; scan results render in the BranchesView. |
| **P13-T3** | Accessibility pass | Audit all views for keyboard navigation (tab order), ARIA labels on interactive elements, focus indicators, and WCAG 2.1 AA contrast ratios. Fix violations. Add `:focus-visible` styles to the design system. | `apps/desktop/src/views/*.vue`, `apps/desktop/src/styles/*.css` | — | yes | 1.5 | Medium | All interactive elements reachable by Tab; all buttons/inputs have ARIA labels; no contrast ratio below 4.5:1 on text; focus indicator visible on all focusable elements. |
| **P13-T4** | SHA256 checksums + signing verification | Automate SHA256SUMS generation in `release.yml` for all release artifacts. Integrate minisign (keys in `.scratch/minisign_keys/`) for signature generation. Add a `verify-release.sh` script that checks checksums + signatures. | `.github/workflows/release.yml`, `scripts/verify-release.sh` | — | yes | 1 | Quick Win | Release workflow produces `SHA256SUMS` + `SHA256SUMS.sig`; `verify-release.sh` validates a downloaded tarball; bad checksum/signature is detected. |
| **P13-T5** | Cross-platform release verification | Add a CI job that downloads the release artifacts for each platform (Linux, macOS, Windows), runs `git-purge --version`, and verifies the binary executes correctly. Tests the "clean container" requirement from R10. | `.github/workflows/release.yml` | P13-T4 | no | 1 | Medium | Post-release CI job runs `git-purge --version` from the downloaded tarball on all 3 platforms; exit code 0; version string matches tag. |
| **P13-T6** | `git-purge show` as separate CLI command | Implement `cmd/show.rs` as a standalone command per CONVENTIONS §9: `git-purge show <ref> [<path>]` displays a tree listing or file content at a given ref. Currently absorbed into `diff`. | `crates/gitpurge-cli/src/cmd/show.rs`, `crates/gitpurge-cli/src/cli.rs`, `crates/gitpurge-cli/src/main.rs` | — | yes | 0.5 | Low | `git-purge show main` lists the tree; `git-purge show main:README.md` prints file content; `--json` produces structured output. |
| **P13-T7** | In-memory task registry for ongoing cleanups | Create a thread-safe registry in `AppState` that tracks running cleanup operations, total/processed counts, and execution logs. Expose Tauri commands to query the list and detailed states. Add a background tasks panel in the Vue sidebar. | `apps/desktop/src-tauri/src/main.rs`, `apps/desktop/src-tauri/src/commands/branch.rs`, `apps/desktop/src/views/Sidebar.vue` | P13-T1 | no | 1.5 | Medium | Navigating away from cleanup view during active deletion does not terminate UI tracking; clicking the task badge in sidebar returns to the progress screen. |
| **P13-T8** | Global toast notification system | Create a reactive Pinia toast store and standard toast container in Vue to display floating success, error, and info alerts. Wire the frontend to automatically map background Tauri notifications to toasts. | `apps/desktop/src/stores/toast.ts`, `apps/desktop/src/components/ToastContainer.vue`, `apps/desktop/src/App.vue` | P13-T1 | yes | 1.0 | Quick Win | Emitting an operation event from the Tauri backend (e.g. mock backup success/fail) displays a corresponding success/error toast floating in the browser corner. |
| **P13-T9** | Custom themed modal dialogs | Replace generic OS-level prompt alerts (`tauri_plugin_dialog`) with custom, beautiful Vue `<ModalDialog />` components utilizing glassmorphism styling, alert icons, and type-to-confirm safety inputs. | `apps/desktop/src/components/ModalDialog.vue`, `apps/desktop/src/views/BranchesView.vue` | P13-T1 | yes | 1.5 | Medium | Triggering deletion prompts opens the glassmorphic custom modal dialog; clicking cancel rejects the action; typing branch names correctly enables the confirm action. |
| **P13-T10** | Disable context menu & shortcuts in production | Restrict standard browser right-click context menu and developer hotkeys (F12, Ctrl+Shift+I, Ctrl+R) in Tauri production builds to prevent diagnostic leakage and accidental reloading. | `apps/desktop/src/main.ts`, `apps/desktop/src-tauri/tauri.conf.json` | — | yes | 0.5 | Quick Win | In production build, right-clicking the window or pressing F12 has no effect; devtools remain enabled in development mode. |
| **P13-T11** | Custom naming convention regex UI input | Add a new text input configuration in `SettingsView.vue` to allow custom regex naming policy definition. If custom regex is provided, validate it. Wire the input to the engine settings DTO and update local/global settings config on save. | `apps/desktop/src/views/SettingsView.vue`, `apps/desktop/src-tauri/src/commands/settings.rs` | P13-T1 | yes | 0.5 | Medium | Custom naming regex field is visible in Settings view; entering an invalid regex shows a validation error; saving updates the repository's naming policy configuration on disk. |

Total ≈ 13.0 ED.

## Exit criteria

- E2e test suite runs headless with ≥5 scenarios passing.
- All Vue views use Pinia stores (no large local reactive state).
- Accessibility audit passes: keyboard nav, ARIA, contrast ratios.
- Release artifacts include SHA256 checksums and minisign signatures.
- Cross-platform verification confirms binary runs on all 3 platforms.

### Requirements & safety invariants satisfied

- **R8** (testing): P13-T1, P13-T2 (e2e suite fulfills P4-T9 acceptance criteria).
- **R9** (tarball + bundles with integrity): P13-T4.
- **R10** (single-binary portable): P13-T5.
- **R11** (release on tag): P13-T4, P13-T5.
- **R12** (minimalist UI, accessible): P13-T3.

## Risks & open questions

- **`tauri-driver` maturity.** tauri-driver for Tauri v2 may have limitations on
  Linux WebKitGTK. Fallback: use Playwright with the webview's debug port.
- **Minisign key management.** Private key must not be committed. Use GitHub Actions
  secrets for CI signing; document key rotation process.
- **WebKitGTK a11y.** Some ARIA patterns may behave differently in WebKitGTK vs
  WebView2/WKWebView. Test on Linux first; document platform-specific exceptions.
