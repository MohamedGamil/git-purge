# P4 — Task Cards

> Phase: **4 — Desktop UI** · Status: **Complete** · Est: 18 ED
> Depends on: P1, P2, P3

---

## P4-T1 · Tauri v2 scaffold + IPC layer ✅ (2026-07-11)

**Goal:** Initialize the Tauri v2 app with Vue 3 + Vite frontend. Set up the IPC layer:
`#[tauri::command]` handlers that call `Engine` methods via managed state.

**Files:** `apps/desktop/src-tauri/src/main.rs`, `apps/desktop/src-tauri/src/commands.rs`,
`apps/desktop/src/ipc.ts`

**Depends on:** P0-T1

**Acceptance:** `pnpm tauri dev` launches the app; a test `invoke("scan")` call round-trips.

---

## P4-T2 · Architecture ban test (UI) ✅ (2026-07-11)

**Goal:** Test that `gitpurge-desktop` does not have `gix`/`git2`/`rusqlite`/`keyring`
in its direct dependencies. Mirrors the CLI architecture guard.

**Files:** `apps/desktop/src-tauri/tests/architecture.rs`

**Depends on:** P4-T1

**Acceptance:** Test passes; adding `git2` to desktop Cargo.toml fails the test.

---

## P4-T3 · Design system + theming ✅ (2026-07-11)

**Goal:** Implement the One Dark Pro + Material design tokens. Light/dark/system theme
switching via `data-theme` attribute and `useTheme` composable.

**Files:** `apps/desktop/src/styles/tokens.css`, `apps/desktop/src/styles/global.css`,
`apps/desktop/src/composables/useTheme.ts`

**Depends on:** P4-T1

**Acceptance:** Theme switch toggles light/dark/system; tokens match
`07-ui-design-system.md`; contrast meets design targets.

---

## P4-T4 · Navigation + layout shell ✅ (2026-07-11)

**Goal:** `App.vue` layout with nav sidebar + `<router-view>`. Vue Router with lazy-loaded
routes for all views.

**Files:** `apps/desktop/src/App.vue`, `apps/desktop/src/router/index.ts`

**Depends on:** P4-T3

**Acceptance:** All routes navigate correctly; nav highlights active view; responsive layout.

---

## P4-T5 · Dashboard + repo management view ✅ (2026-07-11)

**Goal:** Dashboard showing tracked repos with status summary. Add/remove repos.

**Files:** `apps/desktop/src/views/DashboardView.vue`, `apps/desktop/src/stores/repos.ts`

**Depends on:** P4-T4, P4-T1

**Acceptance:** Dashboard lists repos with branch counts; add repo from filesystem picker.

---

## P4-T6 · Branches view (scan + classify) ✅ (2026-07-11)

**Goal:** Branch list with classification badges, filtering, sorting, and selection for
bulk actions.

**Files:** `apps/desktop/src/views/BranchesView.vue`

**Depends on:** P4-T5

**Acceptance:** Branches display with correct classification; filter/sort works; selection
state persists across filter changes.

---

## P4-T7 · Plan preview + execute flow ✅ (2026-07-11)

**Goal:** Plan preview showing actions with "why" reasons. Execute button with dry-run
default, backup-before-destroy, and confirmation dialog.

**Files:** `apps/desktop/src/views/PlanView.vue`

**Depends on:** P4-T6

**Acceptance:** Plan shows correct actions; execute flow backs up first; `SAFE-01` and
`SAFE-04` proven in UI; progress feedback via Tauri events.

---

## P4-T8 · Backup, diff, history, auth, settings views ✅ (2026-07-11)

**Goal:** Remaining views: BackupsView, DiffView, HistoryView, AuthView, SettingsView.

**Files:** `apps/desktop/src/views/*.vue`

**Depends on:** P4-T5

**Acceptance:** All views render correctly and call appropriate Engine methods via IPC.

---

## P4-T9 · e2e smoke + standalone verification ✅ (2026-07-11)

**Goal:** `tauri-driver` + WebDriver smoke suite; Vitest unit tests for stores/components;
verify the app runs with no CLI installed.

**Files:** `apps/desktop/tests/e2e/*.ts`, `apps/desktop/src/**/*.spec.ts`

**Depends on:** P4-T5 through P4-T8

**Acceptance:** e2e smoke passes headless in CI; theme switch e2e green (R12); app runs
with `git-purge` absent from PATH (standalone, ADR-0004).
