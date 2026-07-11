# Definition of Done & Requirements Traceability

`Status: Approved` · `Owner: Program` · `Last-updated: 2026-07-11`

## Global Definition of Done (applies to every phase)
A phase/feature is **Done** only when all of the following hold:

1. **Behavior** — acceptance criteria in the phase doc pass on fixture repos.
2. **Shared core** — the capability exists in `gitpurge-core` and is invoked by the
   CLI (and the UI, if the phase covers UI). No logic duplicated in adapters.
3. **Safety** — dry-run default and backup-before-destroy invariants hold and have
   dedicated regression tests where the phase adds a destructive path.
4. **Tests** — unit + feature tests added; `gitpurge-core` line coverage ≥ 80%
   overall and 100% of the safety invariants; all CI gates green.
5. **Docs** — specs reflect reality; any canonical change has an ADR; user-facing
   help/README updated.
6. **Traceability** — every requirement mapped to the phase (below) has ≥ 1 passing
   test referenced by ID.
7. **No regressions** — full suite green; no new clippy/lint warnings; `cargo deny`
   and `cargo audit` clean.

## Requirements → phases → verification matrix
IDs R1–R12 are defined in [../docs/00-vision-and-scope.md](../docs/00-vision-and-scope.md).

| Req | Phase(s) | Primary verification (test intent) |
| :-- | :-- | :--- |
| **R1** local+remote, view-at-commit | P1 | `show_tree`/file-at-commit returns correct blob at arbitrary SHA on fixtures; remote refs enumerated. |
| **R2** backup before delete, restore, auto-restore, branch/tag, no-force | P2 | Snapshot→delete→restore round-trip; failed-delete triggers offered restore; restore-as-tag; refusing restore leaves state unchanged. |
| **R3** explore/filter/sort/compare/diff | P1, P4 | Filter/sort predicates; `diff(a,b)` matches golden; UI list/compare e2e. |
| **R4** track local+remote, view-at-commit | P1, P8 | Multi-remote fixture; classification stable; `show` parity CLI/UI. |
| **R5** auth methods + secure storage + fallbacks | P6 | Auth via SSH key, HTTPS token; secret stored in keychain fake, absent from logs; system-identity fallback path exercised. |
| **R6** shared abstractions + extensibility | P0, P1 | Architecture test forbids `gix`/`git2`/`rusqlite` in CLI/UI crates; a second fake `GitBackend` swaps in without facade change. |
| **R7** reports + history | P5 | Trend report reproduces legacy progress tables from recorded runs; md/json/html emitted. |
| **R8** unit + feature testing | all | Coverage gate; `nextest` suite; CLI `insta` snapshots; UI Vitest+e2e. |
| **R9** open-source, tarball + bundles | P7 | Release workflow produces tarball + deb/rpm/AppImage/msi/dmg; licenses present. |
| **R10** single binary / portable; restore | P7, P2 | Tarball binary runs with no deps on a clean container; restore semantics per R2. |
| **R11** GH Actions release on tag | P7 | Tag push on a fixture branch yields a draft release with all artifacts + checksums. |
| **R12** minimalist UI, themes, One Dark Pro Material | P4 | Theme switch (light/dark/system) e2e; visual tokens match [../docs/07-ui-design-system.md](../docs/07-ui-design-system.md); a11y smoke. |

## Safety invariants (each has a named regression test — never remove)
- `SAFE-01` mutating commands are dry-run unless `--execute`/explicit confirm.
- `SAFE-02` protected refs (`main`/`master`/`develop`/`staging`/`production`/`HEAD`
  + user list + globs) are never deleted/archived.
- `SAFE-03` tags are never deleted by branch operations.
- `SAFE-04` a verified pre-op snapshot exists before any destructive op unless
  `--no-backup` is explicitly set.
- `SAFE-05` a failed delete offers restore from the pre-op snapshot; declining is a
  no-op.
- `SAFE-06` restore never force-overwrites an existing ref without explicit consent.
- `SAFE-07` no secret material appears in logs, errors, snapshots, or reports.
