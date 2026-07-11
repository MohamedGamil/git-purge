# Phase 4 — Desktop UI (Tauri + Vue)

`Status: Draft` · `Owner: Delivery` · `Last-updated: 2026-07-11` ·
`Related: [ROADMAP](../docs/ROADMAP.md#p4--desktop-ui-18-ed), [CONVENTIONS §10/§12](CONVENTIONS.md), [architecture §4/§7](../docs/02-architecture.md), [ADR-0003](../docs/adr/ADR-0003-ui-vue-tauri.md), [ADR-0004](../docs/adr/ADR-0004-shared-core-embedding.md), [06-ui-spec.md](../docs/06-ui-spec.md), [07-ui-design-system.md](../docs/07-ui-design-system.md), [14-security.md](../docs/14-security.md)`

## Goal

Give Git Purge a face. Build the Tauri v2 desktop shell whose Rust backend
(`gitpurge-desktop`) embeds `gitpurge-core` directly (no sidecar, no CLI process —
ADR-0004) and exposes every capability as a `#[tauri::command]` that mirrors the CLI
verbs ([CONVENTIONS §10](CONVENTIONS.md)). On top of it, build the Vue 3 app: repo
picker, branch explorer (filter/sort/compare/diff), history/commit viewer with
view-at-commit, backups & restore-points browser, the plan → execute review flow,
reports & trend charts, auth manager, and settings — all on the One Dark Pro Material
design system with light/dark/system themes, streamed progress, and cancellation.
The UI reaches full parity with the CLI and runs standalone with no CLI installed.

**Milestone:** M4 — Desktop beta.

**Dependencies:** **P1–P3** merged (read engine, backup/restore, actions + the
`Engine` surface). P5 (reports/trends) and P6 (auth) back the Reports and Auth
screens; per the dependency graph, P4 "folds P5/P6 UI last," so those two screens can
land after P5/P6 merge while the rest of P4 proceeds.

## Tasks

| Task ID | Title | Description | Files (repo-relative) | Depends-on | ∥? | Est (ED) | Acceptance test |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| **P4-T1** | Tauri v2 shell + locked capabilities | Scaffold `gitpurge-desktop` (Tauri v2) initializing a single shared `Engine`; lock down `tauri.conf.json` capabilities so the webview has **no** fs/network beyond commands ([architecture §7](../docs/02-architecture.md), [14-security.md](../docs/14-security.md)); bundle id `com.gitpurge.desktop`. | `apps/desktop/src-tauri/src/{main.rs,lib.rs}`, `apps/desktop/src-tauri/tauri.conf.json`, `apps/desktop/src-tauri/capabilities/*.json` | P3-T3 | no | 2 | App launches to an empty shell; capability audit shows webview cannot touch fs/network directly; `Engine` initializes once; bundle id correct. |
| **P4-T2** | IPC command + event layer | Implement every `#[tauri::command]` from [CONVENTIONS §10](CONVENTIONS.md): `repo_list`, `scan`, `plan`, `backup_create`, `delete_branches`, `archive_branches`, `restore`, `diff`, `show_tree`, `report_generate`, `history_get`, `auth_*`. Each calls `Engine` and returns `Result<T, SerializableError>`. Long ops stream `gitpurge://progress` events and accept a cancellation token. **No git logic in this crate.** | `apps/desktop/src-tauri/src/commands/*.rs`, `apps/desktop/src-tauri/src/events.rs` | P4-T1 | no | 3 | Each command round-trips JSON in an integration test; a long `scan` emits progress events and cancels cleanly; errors serialize as `{code,message,hint}` with **no secrets** (**SAFE-07**); arch ban holds (**R6**). |
| **P4-T3** | Vue 3 app scaffold + typed IPC client | Vite + TS `strict`, Pinia stores, Vue Router 4, `@tauri-apps/api@^2`. Typed `invoke` wrappers per command + a progress/cancel composable + typed error surface. App shell with nav to all screens. | `apps/desktop/src/{main.ts,App.vue}`, `apps/desktop/src/router/index.ts`, `apps/desktop/src/stores/*.ts`, `apps/desktop/src/api/*.ts` | P4-T1 | yes | 2 | `pnpm lint`, `pnpm test` (Vitest), and `vue-tsc --noEmit` pass; a stub screen calls `repo_list` via the typed client and renders the result. |
| **P4-T4** | Design system + themes | Implement design tokens (CSS custom properties) for **One Dark Pro + Material** per [07-ui-design-system.md](../docs/07-ui-design-system.md); light/dark/system theme switching persisted in settings; core primitives (button, table, dialog, toast, badge). | `apps/desktop/src/styles/{tokens.css,themes.css}`, `apps/desktop/src/components/ui/*.vue`, `apps/desktop/src/stores/theme.ts` | P4-T3 | yes | 1.5 | Toggling light/dark/system restyles the app live and persists across restart; token values match the design-system doc; **R12**. |
| **P4-T5** | Repo picker + branch explorer | Repo add/list/remove/show screen; branch explorer listing `Classification`s with filter/sort controls bound to `ActionFilter`/`SortKey`, multi-select, and compare-two → diff. Reuses the same filter/sort semantics as the CLI (R6). | `apps/desktop/src/views/{Repos.vue,BranchExplorer.vue}`, `apps/desktop/src/components/branch/*.vue` | P4-T2, P4-T3, P4-T4 | yes | 2.5 | Explorer filters/sorts a fixture repo identically to the CLI; selecting two branches opens a diff; protected branches are visually flagged and not selectable for destructive actions (**SAFE-02**); **R3**. |
| **P4-T6** | History / commit viewer + view-at-commit | Commit/history viewer and a tree/file viewer rendering `show_tree` (repo/file content **as of any commit**) plus the diff view component. | `apps/desktop/src/views/{HistoryViewer.vue,DiffView.vue,TreeView.vue}` | P4-T2, P4-T3 | yes | 2 | Browsing to an arbitrary commit shows its tree and a selected file's content matching `git-purge show` (CLI/UI parity); diff view matches `diff` output; **R1/R3/R4**. |
| **P4-T7** | Backups browser + plan→execute review flow | Snapshots/restore-points browser (list/show/verify/prune, restore-as-branch/tag with the consent dialog) and the plan → execute flow: preview `Plan` (dry-run) → confirm (stronger confirm for destructive) → `execute` with streamed progress + cancel. | `apps/desktop/src/views/{Backups.vue,PlanReview.vue}`, `apps/desktop/src/components/plan/*.vue` | P4-T2, P4-T3, P4-T4 | yes | 2.5 | UI executes a delete only after explicit confirm; a pre-op snapshot is created and shown first (**SAFE-04**); restore-as-tag works and refuses silent overwrite (**SAFE-06**); progress streams and Cancel aborts (**SAFE-01** preview default). |
| **P4-T8** | Reports/trends + auth manager + settings | Reports screen with trend charts (lightweight SVG) over `report_generate`/`history_get`; auth manager over `auth_*` (add/list/remove/test — secrets never rendered back); settings (policy, backups root, theme). | `apps/desktop/src/views/{Reports.vue,Auth.vue,Settings.vue}`, `apps/desktop/src/components/charts/*.vue` | P4-T2, P4-T3 (report/history need P5; auth needs P6) | yes | 1.5 | Reports render md/json/html export + a trend chart from recorded runs (**R7**); auth manager stores/tests a credential without ever displaying the secret (**R5/SAFE-07**); settings persist. |
| **P4-T9** | e2e smoke + standalone verification | `tauri-driver` + WebDriver smoke suite covering launch, scan, plan-preview, theme switch; Vitest unit coverage for stores/components; verify the app runs with **no CLI installed**. | `apps/desktop/tests/e2e/*.ts`, `apps/desktop/src/**/*.spec.ts` | P4-T5, P4-T6, P4-T7, P4-T8 | no | 1 | e2e smoke passes headless in CI; theme-switch e2e green (**R12**); a run with `git-purge` absent from `PATH` still opens and scans (standalone, ADR-0004); **R8**. |

Total ≈ 18 ED.

## Exit criteria

- Every CLI capability is reachable in the UI; the `tauri-driver` e2e smoke suite
  passes; the app runs standalone without a CLI install (ROADMAP P4 exit).
- The UI calls `Engine` exclusively through the IPC layer; no git/DB/keychain logic in
  `gitpurge-desktop` or the webview (arch ban holds).
- One Dark Pro Material design system with light/dark/system themes; long ops show
  progress and can be cancelled.

### Requirements & safety invariants satisfied

- **R1/R3/R4** (explore/filter/sort/compare/diff/view-at-commit in UI): P4-T5, P4-T6.
- **R5** (auth manager UI): P4-T8. **R7** (reports/trends UI): P4-T8.
- **R6** (shared core, thin UI adapter): P4-T2 (arch ban) + ADR-0004.
- **R8** (Vitest + e2e): P4-T9. **R12** (minimalist UI, themes, One Dark Pro Material):
  P4-T4, P4-T9.
- **SAFE-01** (dry-run/preview default in the review flow): P4-T7. **SAFE-02**
  (protected not selectable): P4-T5. **SAFE-04/05/06** surfaced in the flow: P4-T7.
  **SAFE-07** (no secrets in errors/UI): P4-T2, P4-T8.

## Risks & open questions

- **Tauri v2 capabilities model** is stricter than v1; budget time to get the
  allow-list exactly right so security holds without breaking commands ([14](../docs/14-security.md)).
- **WebView differences** (WebKitGTK vs WebView2 vs WKWebView) can cause rendering/e2e
  flakiness; keep the design system dependency-light and test on the CI matrix.
- **P5/P6 sequencing** — Reports and Auth screens (P4-T8) depend on P5/P6; if those slip,
  ship the rest of P4 with those two screens stubbed behind feature flags and fold them
  in last (per the dependency graph note).
- **Charts without a heavy lib** — trend charts use lightweight SVG; confirm this meets
  the reporting visuals in [10-reporting-and-history.md](../docs/10-reporting-and-history.md).
- **Cancellation semantics** must be safe: cancelling mid-execute must not leave a
  half-applied destructive plan without its snapshot/auto-restore (ties to SAFE-05).
