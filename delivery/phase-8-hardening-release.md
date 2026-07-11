# Phase 8 — Hardening, security review & 1.0

`Status: Draft` · `Owner: Delivery` · `Last-updated: 2026-07-11` ·
`Related: [ROADMAP](../docs/ROADMAP.md#p8--hardening-security-review--10-8-ed), [CONVENTIONS §12/§13](CONVENTIONS.md), [DoD](DEFINITION_OF_DONE.md), [14-security.md](../docs/14-security.md), [12-testing-strategy.md](../docs/12-testing-strategy.md)`

## Goal

Take the feature-complete product to a signed-off 1.0. Run the threat-model /
security review against the [14-security.md](../docs/14-security.md) checklist and a
clean dependency audit; add fuzz/property tests on the policy & classification engines
(the highest-risk correctness surface); do a performance pass on large repos (the
motivating repos had 843 and 2,356 branches); complete the accessibility pass on the
UI and the docs/site; verify coverage and every safety invariant; then cut the 1.0
tag once the Definition of Done holds for every phase.

**Milestone:** M5 — Shippable 1.0.

**Dependencies:** **all** prior phases merged (P0–P7). This phase audits and hardens
what they built; it should add no new features.

## Tasks

| Task ID | Title | Description | Files (repo-relative) | Depends-on | ∥? | Est (ED) | Acceptance test |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| **P8-T1** | Security review + dependency audit | Work the [14-security.md](../docs/14-security.md) threat-model checklist (capability lockdown, no shell-injection surface, secret hygiene, backup/restore integrity); run `cargo-deny` + `cargo-audit` clean; record sign-off. | `docs/14-security.md` (checklist results), `deny.toml`, `SECURITY.md` | P6, P7 | no | 1.5 | Every checklist item is checked or waived-with-rationale; `cargo deny check` and `cargo audit` are clean in CI; secret-hygiene (`SAFE-07`) re-verified on release builds. |
| **P8-T2** | Fuzz + property tests (policy/classification) | `proptest` invariants for age parsing, naming classification, protection resolution, and filter/sort; fuzz targets for URL/ref/name parsing. Assert safety-relevant invariants can never be violated (e.g., a well-known protected name is never classified deletable). | `crates/gitpurge-core/tests/prop_policy.rs`, `crates/gitpurge-core/fuzz/fuzz_targets/*.rs` | P1 | yes | 1.5 | Property suite passes N cases with no shrink-found counterexample; a fuzz target survives a time-boxed run with no panic; `SAFE-02` holds under property testing; **R8**. |
| **P8-T3** | Performance pass on large repos | A `large_repo` fixture (~2,300 branches, matching the motivating repos) and benchmarks for scan/classify/backup; profile hot paths; set and check performance budgets. | `crates/gitpurge-core/benches/*.rs`, `crates/gitpurge-core/src/testkit/builders.rs` (large fixture) | P1, P2 | yes | 1.5 | Scan+classify of the `large_repo` completes within the documented budget with bounded RAM; backup of it stays sub-linear in space (re-checks ADR-0005); no O(n²) blowups. |
| **P8-T4** | Accessibility pass (UI) | Audit and fix the Vue UI for keyboard navigation, focus management, ARIA roles/labels, and color-contrast compliance in light/dark themes ([07-ui-design-system.md](../docs/07-ui-design-system.md)). | `apps/desktop/src/**/*.vue`, `apps/desktop/src/styles/*` | P4 | yes | 1 | Keyboard-only walkthrough reaches every primary action; automated a11y check (axe-style) passes with no critical violations; contrast meets the design-system targets; **R12**. |
| **P8-T5** | Docs & site completion | Fill/finalize the spec docs (04–15) and user-facing docs; publish the docs site / README with install, quickstart, safety model, and CLI/UI references. | `docs/*.md`, `README.md`, `docs/site/*` | P0–P7 | yes | 1.5 | All referenced docs exist and are internally consistent with [CONVENTIONS](CONVENTIONS.md); quickstart reproduces a "clean a repo in <10 min" walkthrough; links resolve. |
| **P8-T6** | Coverage + DoD traceability sign-off | Verify `gitpurge-core` line coverage ≥ 80% and 100% of the safety invariants; fill the [DoD](DEFINITION_OF_DONE.md) R-matrix with the passing test per requirement; confirm every phase's exit criteria are met. | `.github/workflows/ci.yml` (coverage gate), `delivery/DEFINITION_OF_DONE.md` (verified matrix) | P0–P7 | no | 0.5 | Coverage gate ≥ 80% enforced in CI; each `SAFE-01..07` maps to a named passing test; each R1–R12 row cites a green test. |
| **P8-T7** | 1.0 release cut | Finalize `CHANGELOG`, bump versions, and cut the `v1.0.0` tag driving the P7 release workflow; confirm the produced release passes the P7 verification. | `CHANGELOG.md`, `Cargo.toml` (versions), `apps/desktop/src-tauri/tauri.conf.json` (version) | P8-T1, P8-T2, P8-T3, P8-T4, P8-T5, P8-T6 | no | 0.5 | Tagging `v1.0.0` yields a verified release (all artifacts + checksums + signatures); the security checklist is signed off; DoD holds for every phase (ROADMAP P8 exit). |

Total ≈ 8 ED.

## Exit criteria

- Security review checklist signed off; `cargo-deny`/`cargo-audit` clean; the DoD is
  met for every phase (ROADMAP P8 exit).
- Property/fuzz tests guard policy & classification; performance budgets hold on a
  2,300-branch fixture; the UI passes the accessibility pass.
- Coverage ≥ 80% on `gitpurge-core` with 100% of safety invariants covered; the R-matrix
  is fully green; `v1.0.0` is released via the P7 workflow.

### Requirements & safety invariants satisfied

- **R4/R8** (stable classification at scale; testing): P8-T2, P8-T3, P8-T6.
- **R5/R9/R11** re-verified on release builds (secret hygiene, artifacts, signed
  release): P8-T1, P8-T7.
- **R12** (accessibility, themes): P8-T4.
- **SAFE-01..07** — all re-verified end to end and mapped to named tests in the DoD
  matrix (P8-T6); `SAFE-02` additionally property-tested (P8-T2); `SAFE-07`
  re-verified on release builds (P8-T1).

## Risks & open questions

- **"Signed off by whom"** — with no server/multi-user mode, define who owns the
  security sign-off (maintainer self-review vs. external audit) and record it in
  `SECURITY.md`.
- **Performance budgets are unset** — P8-T3 must first *establish* budgets from a
  baseline measurement, then enforce them; treat the first run as calibration.
- **Docs 04–15 are large** — several spec docs are referenced throughout but not yet
  written; P8-T5 is the backstop, but earlier phases should fill their own spec doc as
  they build (avoid a doc cliff at the end).
- **Notarization/signing gaps from P7** may surface here as release blockers; ensure the
  cert/secret story is settled before the 1.0 tag.
- **Fuzz flakiness in CI** — time-box fuzz runs and keep a corpus so CI is deterministic;
  run longer fuzzing out-of-band.
