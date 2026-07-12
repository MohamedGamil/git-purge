# Agent Guide — Implementing Git Purge

`Status: Approved` · `Owner: Program` · `Last-updated: 2026-07-11`

This guide is for any engineer or AI agent implementing a task. Follow it exactly so
that dozens of independent sessions produce one coherent codebase.

## 0. Before you touch code
1. Read [`BATON.md`](BATON.md) — check for Active tasks first. If none, promote
   2–5 tasks from the Queue.
2. Read [`CONVENTIONS.md`](CONVENTIONS.md) end to end. It overrides everything.
3. Read the phase doc for your tasks and the spec doc(s) they reference in
   [`../docs/`](../docs/).
4. Confirm your task's **dependencies** (other tasks) are already in Completed
   on the baton. If not, stop and report the blocker — do not stub around a
   missing dependency silently.

## 1. Golden rules (non-negotiable)
- **Never put git/DB/keychain logic in the CLI or Tauri crates.** It goes in
  `gitpurge-core` behind a port. CLI/UI only translate + render.
- **Dry-run is the default** for any mutating path. Mutations need explicit opt-in.
- **No destructive op without a prior verified backup** (unless `--no-backup` is
  explicitly set). If you add a destructive path, you add the backup wrapper and a
  test proving it.
- **No secrets in logs, errors, snapshots, or reports.** Ever.
- **No hardcoded paths, repos, users, or ages.** Everything is config/args (the
  legacy scripts' `/home/mgamil/...` hardcoding is the anti-pattern we're removing).
- **No network or machine-specific state in tests.** Use `testkit` fixture repos.

## 2. Environment setup
```bash
# Rust
rustup toolchain install stable        # rust-toolchain.toml pins the version
cargo install cargo-nextest cargo-deny cargo-insta

# Frontend (only if working on apps/desktop)
corepack enable && corepack prepare pnpm@9 --activate
cd apps/desktop && pnpm install

# Tauri prerequisites per OS: see docs/13-distribution-and-ci.md
```

## 3. Workflow for a task
1. Branch: `feat/P<phase>-T<n>-<slug>` (Conventional Commits for messages).
2. Implement the smallest slice that satisfies the task's acceptance test.
3. Add/extend tests **in the same change** (unit + at least one feature test).
4. Run the local gate (must all pass):
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo nextest run --all
   cargo deny check
   # frontend tasks additionally:
   pnpm -C apps/desktop lint && pnpm -C apps/desktop test && pnpm -C apps/desktop exec vue-tsc --noEmit
   ```
5. Update docs if behavior/decisions changed. Canonical changes ⇒ add/adjust an ADR.
6. Open a PR titled `P<phase>-T<n>: <summary>`; fill the PR checklist (below).

## 4. PR checklist (paste into every PR)
```
- [ ] Conforms to CONVENTIONS.md (names, versions, layout, safety model)
- [ ] Logic lives in gitpurge-core; CLI/UI are thin
- [ ] Dry-run default preserved; destructive paths backed up first
- [ ] Unit tests + ≥1 feature test added; all gates green locally
- [ ] No secrets/PII in logs, errors, or fixtures
- [ ] Docs/ADR updated if decisions or behavior changed
- [ ] Requirement(s) R__ referenced; DoD row(s) satisfied
```

## 5. Definition of "done" for a task
- Acceptance test in the phase doc passes and is committed.
- The public API you added is documented (`///`) and used by at least the CLI (and
  UI if in scope), proving the shared-core contract.
- No new `clippy` warnings, no `unwrap()`/`expect()` in runtime library paths.

## 6. Reporting back (for AI agent sessions)
End your session with:
1. **Update [`BATON.md`](BATON.md):** Move completed tasks Active → Completed.
   Defer unfinished tasks back to Queue with a reason.
2. Short structured report:
   - Task ID + one-line outcome.
   - Files created/modified.
   - Tests added and how to run them.
   - Anything that deviated from the spec (and why), or new blockers discovered.
   - Follow-up tasks you recommend (with proposed IDs).

Return findings/decisions, **not** raw file dumps — the orchestrator keeps the
conclusion, not the diff.

## 7. When specs are ambiguous
Prefer the safest, simplest interpretation consistent with CONVENTIONS + the safety
model, implement it, and note the assumption in your report. Do not block on
questions you can answer with a sensible default. Escalate only true forks
(irreversible or user-preference decisions).
