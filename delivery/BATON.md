# Delivery Baton

> **Last updated:** 2026-07-17 · **Session:** 49 · **Updated by:** Implementation loop (orient + implement + document)

The baton is the **single source of truth** for what's done, what's in progress,
and what's next. Every implementation session **reads the baton first** and
updates it last. This prevents sessions from chewing on too many tasks or
duplicating work.

## Rules

1. **One active batch at a time.** A session picks up the `Active` tasks (max 3–5).
   No new tasks are started until the active batch is complete or explicitly deferred.
2. **Active → Done on completion.** When a task passes verification, move it from
   `Active` to the top of `Completed` (newest first).
3. **Queue → Active on pickup.** At the start of a session, if there are no Active
   tasks, promote the next 2–5 tasks from `Queue` based on the priority rules in
   the phase doc and dependency order.
4. **Never skip the baton update.** The DOCUMENT step of every session ends by
   updating this file.
5. **Blocked tasks stay in Queue** with a `⛔ blocked by <task-id>` note.

---

## ✅ Completed

> Newest first. Only the last ~15 entries are kept here; older entries live in
> the walkthrough index and task cards.

| Task ID | Title | Session | Date |
|:--------|:------|:--------|:-----|
| P13-T8 | Global toast notification system | 49 | 2026-07-17 |
| P13-T9 | Custom themed modal dialogs | 49 | 2026-07-17 |
| P13-T11 | Custom naming convention regex UI input | 49 | 2026-07-17 |
| P13-T1 | Pinia stores for remaining views | 48 | 2026-07-17 |
| P13-T4 | SHA256 checksums + signing verification | 48 | 2026-07-17 |
| P13-T10 | Disable context menu & shortcuts in production | 48 | 2026-07-17 |
| P13-T6 | `git-purge show` as separate CLI command | 48 | 2026-07-17 |
| P12-T3 | `history import --legacy-json` | 47 | 2026-07-17 |
| P12-T5 | Performance benchmark suite | 47 | 2026-07-17 |
| P12-T7 | `SECURITY.md` threat-model content | 47 | 2026-07-17 |
| P12-T8 | Multi-threaded branch delete/archive with progress reporting | 47 | 2026-07-17 |
| P10-T7 | Expand CI to cross-platform matrix | 46 | 2026-07-17 |
| P12-T1 | Implement trend diffs | 46 | 2026-07-17 |
| P12-T2 | `insta` golden tests for report renderers | 46 | 2026-07-17 |
| P12-T4 | `proptest` property tests for policy engine | 46 | 2026-07-17 |
| P10-T3 | Split Tauri `commands.rs` into domain modules | 45 | 2026-07-17 |
| P11-T7 | Wire auth CLI commands to real backends | 44 | 2026-07-17 |

---

## 🔄 Active

> Tasks currently being worked on. Max 3–5 per session.

| Task ID | Title | Phase | Assignee | Notes |
|:--------|:------|:------|:---------|:------|
| — | — | — | — | *No active tasks. Next session should promote from Queue.* |

---

## 📋 Queue

> Ordered by: phase priority (critical first), then within-phase task order
> (dependencies → quick wins → high → medium → low).

### High — Phase 10: Structural Consolidation & Spec Alignment

| # | Task ID | Title | Est | Priority | Blocked by |
|:--|:--------|:------|:----|:---------|:-----------|

### High — Phase 12: Reporting Completion & Hardening Foundations

| # | Task ID | Title | Est | Priority | Blocked by |
|:--|:--------|:------|:----|:---------|:-----------|
| 10 | P12-T6 | Frontend Vitest expansion | 3 ED | Medium | — |

### Medium — Phase 13: Desktop Polish & Packaging Finalization

| # | Task ID | Title | Est | Priority | Blocked by |
|:--|:--------|:------|:----|:---------|:-----------|
| 15 | P13-T7 | In-memory task registry for ongoing cleanups | 1.5 ED | Medium | — |
| 20 | P13-T2 | `tauri-driver` + WebDriver e2e smoke suite | 2 ED | High | — |
| 21 | P13-T3 | Accessibility pass | 1.5 ED | Medium | — |
| 22 | P13-T5 | Cross-platform release verification | 1 ED | Medium | — |
