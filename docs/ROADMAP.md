# Roadmap

`Status: Approved` · `Owner: Program` · `Last-updated: 2026-07-11` ·
`Related: ../delivery/, 02-architecture.md`

Estimates are in **engineer-days** (ED) for one focused engineer (or one capable
agent session-equivalent). They are planning figures, not commitments. Phases are
ordered by dependency; several tasks *within* a phase parallelize across agents.

## Milestones (outcomes, not dates)

| Milestone | Definition | Phases |
| :-- | :--- | :--- |
| **M0 – Foundations ready** | Workspace builds, CI green, core skeleton + ports compile with fakes. | P0 |
| **M1 – Read-only insight** | `scan`/`plan`/`diff`/`show`/`report` work on real repos, read-only. | P1, P5(partial) |
| **M2 – Safe mutation** | Backup, delete, archive, restore all work with full safety net. | P2, P3(delete/archive) |
| **M3 – CLI 1.0** | Complete, tested CLI incl. auth, history, reports, completions. | P3, P5, P6 |
| **M4 – Desktop beta** | Tauri+Vue app with full parity to CLI, One Dark Pro UI. | P4 |
| **M5 – Shippable 1.0** | Packaged for all targets, signed, released via CI, docs done. | P7, P8 |

## Phase overview

| Phase | Title | Est (ED) | Depends on | Delivery doc |
| :-- | :--- | :--: | :--- | :--- |
| **P0** | Foundations & scaffolding | 6 | — | [phase-0-foundations.md](../delivery/phase-0-foundations.md) |
| **P1** | Core read engine (git + classify + diff) | 12 | P0 | [phase-1-core-read.md](../delivery/phase-1-core-read.md) |
| **P2** | Backup & restore | 10 | P1 | [phase-2-backup-restore.md](../delivery/phase-2-backup-restore.md) |
| **P3** | Actions + CLI (delete/archive/restore) | 12 | P1, P2 | [phase-3-cli.md](../delivery/phase-3-cli.md) |
| **P4** | Desktop UI (Tauri + Vue) | 18 | P1–P3 | [phase-4-ui.md](../delivery/phase-4-ui.md) |
| **P5** | Reporting & history/trends | 8 | P1 | [phase-5-reporting.md](../delivery/phase-5-reporting.md) |
| **P6** | Authentication & secure storage | 9 | P1 | [phase-6-auth.md](../delivery/phase-6-auth.md) |
| **P7** | Packaging, distribution & CI release | 9 | P3, P4 | [phase-7-packaging-ci.md](../delivery/phase-7-packaging-ci.md) |
| **P8** | Hardening, security review & 1.0 | 8 | all | [phase-8-hardening-release.md](../delivery/phase-8-hardening-release.md) |
| | **Total** | **~92 ED** | | |

At ~92 ED, a solo engineer lands ≈ 18–20 calendar weeks; a 3-person team with the
parallelizable phases (P5/P6 alongside P3/P4) lands ≈ 8–10 weeks.

## Dependency graph

```
P0 ──> P1 ──┬──> P2 ──> P3 ──┬──────────────> P7 ──> P8
            │                │
            ├──> P5 ─────────┤
            ├──> P6 ─────────┤
            └──────> P4 ─────┘   (P4 needs P1–P3 core surface; folds P5/P6 UI last)
```

## Phase detail

### P0 — Foundations & scaffolding (6 ED)
Cargo workspace, three crates compiling, `rust-toolchain`, lint/format/deny gates,
`ci.yml`, `error.rs`, `config.rs`, all **ports as traits with in-memory fakes**, the
`testkit` fixture-repo builder, and doc/ADR skeletons. **Exit:** `cargo build`,
`cargo clippy -D warnings`, `cargo nextest run` all green in CI; fakes prove the
layering.

### P1 — Core read engine (12 ED)
`GitBackend` (gix primary, git2 fallback), ref enumeration, commit/date/author
metadata, merged/ancestry checks, age & naming classification (port the
`generate_reports.py` logic + regex), filters/sort, `diff` between refs, and
`show_tree`/file-at-commit (Requirement 1/4). **Exit:** `scan`/`plan`/`diff` return
correct data on fixture repos; classification matches golden snapshots.

### P2 — Backup & restore (10 ED)
Bare-mirror-per-repo with namespaced snapshot refs (minimal space), snapshot
metadata, `create/list/show/verify/prune`, restore-as-branch and restore-as-tag with
consent, and the auto-restore-on-failed-delete wrapper (Requirement 2/10). **Exit:**
snapshot → delete → verify-gone → restore round-trips byte-identically; space stays
sub-linear across snapshots.

### P3 — Actions + CLI (12 ED)
`delete` (merged/stale + optional unmerged), `archive` (merge to legacy via
ours/theirs), `restore`, wired through the full safety model. Complete `clap` CLI:
all verbs from CONVENTIONS §9, `--json`, dry-run default, confirmations, exit codes,
completions, `install-cli`. **Exit:** `assert_cmd`+`insta` snapshot suite passes;
CLI reproduces every capability of the original bash scripts.

### P4 — Desktop UI (18 ED)
Tauri v2 shell, IPC command layer over `Engine`, Vue 3 app: repo picker, branch
explorer (filter/sort/compare/diff), history/commit viewer, backups & restore-points
browser, plan/execute review flow, reports & trend charts, auth manager, settings.
One Dark Pro Material design system, light/dark/system themes, progress+cancel.
**Exit:** every CLI capability reachable in the UI; e2e (`tauri-driver`) smoke suite
passes; runs standalone without a CLI install.

### P5 — Reporting & history/trends (8 ED)
SQLite history store, run recording, trend diffs vs previous+baseline (port
`track_branch_trends.py`), report generation in md/json/html, exports. **Exit:**
trend report reproduces the legacy progress-report tables from recorded runs.

### P6 — Authentication & secure storage (9 ED)
Credential providers (SSH key, HTTPS user/pass, token), OS-keychain storage via
`keyring` + encrypted-file fallback, system SSH-agent/identity fallback,
user-provided keys/identities, `auth test` (Requirement 5). **Exit:** authenticated
fetch/push against HTTPS-token and SSH remotes; secrets never touch logs/plaintext.

### P7 — Packaging, distribution & CI release (9 ED)
Self-contained tarball per platform + `install-cli`, Tauri bundles (deb/rpm/AppImage/
msi/nsis/dmg), `release.yml` matrix on tag → GitHub Release with checksums &
signatures (Requirement 9/11). **Exit:** pushing a `vX.Y.Z` tag yields a full release
with all artifacts and verified checksums.

### P8 — Hardening, security review & 1.0 (8 ED)
Threat-model review, dependency audit (`cargo-deny`/`cargo-audit`), fuzz/property
tests on policy & classification, docs/site, accessibility pass on UI, performance
pass on large repos, 1.0 tag. **Exit:** security review checklist signed off;
DoD met for every phase.

## Cross-cutting acceptance (traceability to requirements)
Each requirement (R1–R12 in [00-vision-and-scope.md](00-vision-and-scope.md)) maps to
phase exit criteria and a test. The [delivery DoD](../delivery/DEFINITION_OF_DONE.md)
holds the full traceability matrix; no phase is "done" until its mapped requirements
have a passing test.
