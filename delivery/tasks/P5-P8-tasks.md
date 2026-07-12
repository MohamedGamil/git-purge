# P5 — Task Cards

> Phase: **5 — Reporting & history** · Status: **Not started** · Est: 8 ED
> Depends on: P1

---

## P5-T1 · SQLite history store adapter ✅ (2026-07-11)

**Goal:** Implement `HistoryStore` trait with SQLite. DDL for `repos`, `runs`, `metrics`,
`branch_snapshots`, `snapshots` tables. Dedup by `metrics_hash`.

**Files:** `crates/gitpurge-core/src/history/sqlite.rs`

**Depends on:** P0-T4

**Acceptance:** Round-trip: record a `RunReport` → query → identical; duplicate inserts
are idempotent.

**Completed:** 2026-07-11. SQLite adapter fully implemented and tested.

---

## P5-T2 · Report generation (md/json/html) ✅ (2026-07-11)

**Goal:** Implement `ReportSink` adapters for markdown, JSON, and HTML output. Port all 6
audit sections from `generate_reports.py`.

**Files:** `crates/gitpurge-core/src/report/markdown.rs`, `crates/gitpurge-core/src/report/json.rs`,
`crates/gitpurge-core/src/report/html.rs`

**Depends on:** P1-T4

**Acceptance:** Markdown report matches legacy format; JSON is machine-parseable; HTML
renders correctly in a browser.

**Completed:** 2026-07-11. All 3 report formats implemented.

---

## P5-T3 · Trend history + legacy import

**Goal:** Trend queries from the history store. Port `track_branch_trends.py` trend tables.
`history import --legacy-json` migration path.

**Files:** `crates/gitpurge-core/src/history/trends.rs`

**Depends on:** P5-T1

**Acceptance:** Trend report reproduces legacy progress tables; legacy JSON import
populates the SQLite database correctly.

---

## P5-T4 · Engine wiring + report tests

**Goal:** Wire `Engine::report`, `Engine::history`. Test suite with fixture data.

**Files:** `crates/gitpurge-core/src/lib.rs`

**Depends on:** P5-T1, P5-T2, P5-T3

**Acceptance:** `report` on fixture data produces correct output in all 3 formats;
`history` shows trend data; R7 satisfied.

---

# P6 — Task Cards

> Phase: **6 — Authentication** · Status: **Not started** · Est: 8 ED
> Depends on: P1

---

## P6-T1 · OS keychain adapter

**Goal:** Implement `SecretStore` using the `keyring` crate for OS keychain (macOS
Keychain, Windows Credential Manager, Linux Secret Service).

**Files:** `crates/gitpurge-core/src/auth/keychain.rs`

**Depends on:** P0-T4

**Acceptance:** Store/retrieve/delete a credential via OS keychain; works on all 3 platforms.

---

## P6-T2 · Encrypted file fallback

**Goal:** Implement `SecretStore` fallback using an encrypted file for environments without
a keychain (headless servers, containers).

**Files:** `crates/gitpurge-core/src/auth/encrypted_file.rs`

**Depends on:** P0-T4

**Acceptance:** Credential stored/retrieved from encrypted file; SAFE-07 (no secrets in
plaintext) proven.

---

## P6-T3 · Auth method resolution

**Goal:** Auto-detect and resolve auth method: SSH key, HTTPS token, username/password.
Wire into `GitBackend` credential callbacks.

**Files:** `crates/gitpurge-core/src/auth/resolver.rs`

**Depends on:** P6-T1, P6-T2

**Acceptance:** SSH key auth works for push; HTTPS token auth works; system-identity
fallback exercised; `SAFE-07` re-verified.

---

## P6-T4 · Engine wiring + auth tests

**Goal:** Wire `Engine` auth methods. Test suite with fake secret store.

**Files:** `crates/gitpurge-core/src/lib.rs`

**Depends on:** P6-T1, P6-T2, P6-T3

**Acceptance:** Auth via SSH key, HTTPS token exercised; secrets never appear in logs;
R5 satisfied.

---

# P7 — Task Cards

> Phase: **7 — Packaging & CI** · Status: **Not started** · Est: 8 ED
> Depends on: P3, P4

---

## ✅ P7-T1 · Release workflow (GitHub Actions) [2026-07-13]

**Goal:** `release.yml` triggered on `v*` tag: matrix build of CLI tarballs
(linux gnu+musl, windows-msvc, macOS universal) + Tauri bundles; SHA256 checksums;
draft GitHub Release.

**Files:** `.github/workflows/release.yml`

**Depends on:** P3, P4

**Acceptance:** Tag push produces draft release with all artifacts + checksums; R11 satisfied.

---

## P7-T2 · Portable tarball packaging ✅ (2026-07-12)

**Goal:** CLI binary tarball with README, LICENSE, completions. Zero-setup: extract and run.

**Files:** `scripts/package-tarball.sh`

**Depends on:** P7-T1

**Acceptance:** Tarball binary runs on clean container with no deps; R10 satisfied.

**Completed:** 2026-07-12. Package script is integrated at `ci/package-tarball.sh`.

---

## P7-T3 · Tauri bundle configuration

**Goal:** Configure Tauri bundler for all platforms: `.deb`, `.rpm`, `.AppImage`, `.msi`, `.dmg`.

**Files:** `apps/desktop/src-tauri/tauri.conf.json`

**Depends on:** P4-T1

**Acceptance:** Tauri build produces installable bundle on each platform.

---

## P7-T4 · install-cli implementation ✅ (2026-07-12)

**Goal:** `git-purge install-cli --user/--system` copies binary to appropriate PATH
location per OS.

**Files:** `crates/gitpurge-cli/src/cmd/install.rs`

**Depends on:** P3-T9

**Acceptance:** `install-cli --user` places binary on PATH; `git purge` works as subcommand.

**Completed:** 2026-07-12. Command implemented in `install_cli.rs`.

---

# P8 — Task Cards

> Phase: **8 — Hardening & 1.0** · Status: **Not started** · Est: 8 ED
> Depends on: P0–P7

---

## P8-T1 · Security review + dependency audit

**Goal:** Work the 14-security.md threat-model checklist. Run `cargo-deny` + `cargo-audit`
clean. Record sign-off.

**Files:** `docs/14-security.md`, `deny.toml`, `SECURITY.md`

**Depends on:** P6, P7

**Acceptance:** Every checklist item checked or waived-with-rationale; `cargo deny check`
and `cargo audit` clean in CI; SAFE-07 re-verified on release builds.

---

## P8-T2 · Fuzz + property tests

**Goal:** `proptest` invariants for age parsing, naming classification, protection
resolution, filter/sort. Fuzz targets for URL/ref/name parsing.

**Files:** `crates/gitpurge-core/tests/prop_policy.rs`, `crates/gitpurge-core/fuzz/`

**Depends on:** P1

**Acceptance:** Property suite passes with no counterexample; fuzz target survives
time-boxed run; SAFE-02 holds under property testing.

---

## P8-T3 · Performance pass

**Goal:** `large_repo` fixture (~2,300 branches) + benchmarks for scan/classify/backup.

**Files:** `crates/gitpurge-core/benches/*.rs`

**Depends on:** P1, P2

**Acceptance:** Scan+classify within budget; backup stays sub-linear in space; no O(n²).

---

## P8-T4 · Accessibility pass (UI)

**Goal:** Keyboard navigation, focus management, ARIA roles/labels, color-contrast audit.

**Files:** `apps/desktop/src/**/*.vue`

**Depends on:** P4

**Acceptance:** Keyboard-only walkthrough; automated a11y check passes; contrast meets
targets; R12 satisfied.

---

## P8-T5 · Docs & site completion

**Goal:** Finalize spec docs; publish user-facing docs with install, quickstart, safety
model, and CLI/UI references.

**Files:** `docs/*.md`, `README.md`

**Depends on:** P0–P7

**Acceptance:** All docs exist and are internally consistent; quickstart reproduces a
"clean a repo in <10 min" walkthrough; links resolve.

---

## P8-T6 · Coverage + DoD traceability sign-off

**Goal:** Verify `gitpurge-core` line coverage ≥ 80%; 100% of safety invariants covered;
fill the DoD R-matrix with passing tests per requirement.

**Files:** `.github/workflows/ci.yml`, `delivery/DEFINITION_OF_DONE.md`

**Depends on:** P0–P7

**Acceptance:** Coverage gate ≥ 80%; each SAFE-01..07 maps to a named passing test;
each R1–R12 row cites a green test.

---

## P8-T7 · 1.0 release cut

**Goal:** Finalize CHANGELOG, bump versions, cut `v1.0.0` tag.

**Files:** `CHANGELOG.md`, `Cargo.toml`, `apps/desktop/src-tauri/tauri.conf.json`

**Depends on:** P8-T1 through P8-T6

**Acceptance:** `v1.0.0` yields verified release with all artifacts + checksums +
signatures; security checklist signed off; DoD holds for every phase.
