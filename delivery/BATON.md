# Delivery Baton

> **Last updated:** 2026-07-17 · **Session:** 40 · **Updated by:** Implementation loop (complete)

The baton is the **single source of truth** for what's done, what's in progress,
and what's next. Every implementation session **reads the baton first** and
**updates it last**. This prevents sessions from chewing on too many tasks or
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
| P9-T2 | Core integration test directory | 40 | 2026-07-17 |
| P9-T4 | `insta` snapshot tests for CLI | 40 | 2026-07-17 |
| P11-T1 | Credential model types | 40 | 2026-07-17 |
| P10-T4 | Resolve license inconsistency | 40 | 2026-07-17 |
| P10-T5 | Update CONVENTIONS/AGENTS.md for ADR-0006 | 40 | 2026-07-17 |
| P7-T1 | Debug and stabilize release workflow | 37 | 2026-07-13 |
| P9-T1 | Named safety regression tests (SAFE-01–07) | 37 | 2026-07-13 |
| P9-T3 | Desktop architecture guard test | 37 | 2026-07-13 |
| P9-T6 | Remove `cmd/stubs.rs` or implement | 37 | 2026-07-13 |
| P9-T8 | Reconcile delivery task cards | 37 | 2026-07-13 |
| P4-T9 | Desktop e2e + standalone | 05 | 2026-07-11 |
| P4-T8 | Remaining views | 05 | 2026-07-11 |
| P4-T7 | Plan + execute flow | 05 | 2026-07-11 |
| P4-T6 | Branches view | 05 | 2026-07-11 |
| P4-T5 | Dashboard + repo mgmt | 05 | 2026-07-11 |
| P4-T4 | Navigation + layout | 04 | 2026-07-11 |

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

### Critical — Phase 9: Safety Retrofit & Testing Debt

| # | Task ID | Title | Est | Priority | Blocked by |
|:--|:--------|:------|:----|:---------|:-----------|
| 1 | P9-T2 | Core integration test directory | 2 ED | Quick Win | — |
| 2 | P9-T4 | `insta` snapshot tests for CLI | 2 ED | High | — |
| 3 | P9-T5 | Coverage gate in CI | 0.5 ED | High | P9-T1, P9-T2 |
| 4 | P9-T7 | Exit-code stability tests | 0.5 ED | Medium | P9-T4 |

### Critical — Phase 11: Authentication & Credential Management

| # | Task ID | Title | Est | Priority | Blocked by |
|:--|:--------|:------|:----|:---------|:-----------|
| 9 | P11-T1 | Credential model types | 0.5 ED | Quick Win | — |
| 10 | P11-T6 | `safe_07` secret hygiene regression suite | 1 ED | Quick Win | P11-T1 |
| 11 | P11-T2 | Keyring `SecretStore` adapter | 1.5 ED | High | P11-T1 |
| 12 | P11-T3 | Encrypted file `SecretStore` fallback | 2 ED | High | P11-T1 |
| 13 | P11-T4 | Credential resolver chain | 1 ED | High | P11-T2, P11-T3 |
| 14 | P11-T5 | git2 credential callback bridge | 1.5 ED | High | P11-T4 |
| 15 | P11-T7 | Wire auth CLI commands to real backends | 1 ED | Medium | P11-T4 |

### High — Phase 10: Structural Consolidation & Spec Alignment

| # | Task ID | Title | Est | Priority | Blocked by |
|:--|:--------|:------|:----|:---------|:-----------|
| 16 | P10-T4 | Resolve license inconsistency | 0.25 ED | Quick Win | — |
| 17 | P10-T5 | Update CONVENTIONS/AGENTS.md for ADR-0006 | 0.25 ED | Quick Win | — |
| 18 | P10-T1 | Extract delete action into `action/delete.rs` | 0.5 ED | Quick Win | ⛔ P9-T1 |
| 19 | P10-T6 | Amend ADR-0002 or build ShellGitBackend | 0.25–1.5 ED | Quick Win or High | — |
| 20 | P10-T2 | Break up `lib.rs` into Engine modules | 2.5 ED | High | ⛔ P10-T1 |
| 21 | P10-T3 | Split Tauri `commands.rs` into domain modules | 1 ED | Medium | — |
| 22 | P10-T7 | Expand CI to cross-platform matrix | 1 ED | Medium | — |

### High — Phase 12: Reporting Completion & Hardening Foundations

| # | Task ID | Title | Est | Priority | Blocked by |
|:--|:--------|:------|:----|:---------|:-----------|
| 23 | P12-T1 | Implement trend diffs | 1.5 ED | Quick Win | — |
| 24 | P12-T2 | `insta` golden tests for report renderers | 1 ED | Quick Win | — |
| 25 | P12-T4 | `proptest` property tests for policy engine | 1.5 ED | High | — |
| 26 | P12-T3 | `history import --legacy-json` | 1 ED | Medium | ⛔ P12-T1 |
| 27 | P12-T5 | Performance benchmark suite | 1.5 ED | Medium | — |
| 28 | P12-T6 | Frontend Vitest expansion | 3 ED | Medium | — |
| 29 | P12-T7 | `SECURITY.md` threat-model content | 1 ED | Medium | — |

### Medium — Phase 13: Desktop Polish & Packaging Finalization

| # | Task ID | Title | Est | Priority | Blocked by |
|:--|:--------|:------|:----|:---------|:-----------|
| 30 | P13-T4 | SHA256 checksums + signing verification | 1 ED | Quick Win | — |
| 31 | P13-T1 | Pinia stores for remaining views | 2 ED | Quick Win | — |
| 32 | P13-T2 | `tauri-driver` + WebDriver e2e smoke suite | 2 ED | High | ⛔ P13-T1 |
| 33 | P13-T3 | Accessibility pass | 1.5 ED | Medium | — |
| 34 | P13-T5 | Cross-platform release verification | 1 ED | Medium | ⛔ P13-T4 |
| 35 | P13-T6 | `git-purge show` as separate CLI command | 0.5 ED | Low | — |
