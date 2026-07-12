# Delivery

This directory turns the [specs](../docs/) into **agent-runnable work**. It is the
operational plan for building Git Purge across many focused engineer/agent sessions.

## Contents
- [`BATON.md`](BATON.md) — **read first every session.** Tracks completed, active,
  and queued tasks. The single source of truth for "what do I work on next?"
- [`CONVENTIONS.md`](CONVENTIONS.md) — canonical decisions. Source of truth for
  names, versions, layout, and safety model.
- [`AGENT_GUIDE.md`](AGENT_GUIDE.md) — how an implementation agent should work:
  setup, workflow, guardrails, how to pick up a task, how to report done.
- [`DEFINITION_OF_DONE.md`](DEFINITION_OF_DONE.md) — global DoD + R1–R12
  traceability matrix.
- `phase-0-foundations.md` … `phase-8-hardening-release.md` — original delivery
  phases (P0–P8), each a checklist of tasks with IDs, acceptance criteria, and
  file targets.
- `phase-9-13-retrofit.md` — retrofit phases (P9–P13) to close gaps identified
  in the [project analysis](../.scratch/analysis/delivery-vs-specs-analysis.md).
- [`tasks/`](tasks/) — expanded task cards for each phase.

## How to use (for a human orchestrator or a lead agent)
1. **Read [`BATON.md`](BATON.md)** — see what's completed, what's active, and
   what's next in the queue. This is the starting point for every session.
2. Read [`CONVENTIONS.md`](CONVENTIONS.md) and the relevant [`../docs/`](../docs/)
   spec for the tasks you're picking up.
3. **Promote 2–5 tasks** from the Queue to Active in the baton. Prefer tasks whose
   dependencies are met, following the priority order in the phase doc.
4. Each agent follows [`AGENT_GUIDE.md`](AGENT_GUIDE.md) and closes tasks against
   the [DoD](DEFINITION_OF_DONE.md).
5. **Update the baton** at the end of every session: move completed tasks to
   Completed, clear Active if done, note blockers.
6. A phase is complete only when its **Exit criteria** (in the phase doc) and all
   mapped requirements pass tests.

## Task ID scheme
`P<phase>-T<n>` (e.g. `P1-T3`). Sub-tasks: `P1-T3.a`. Each task lists: goal, files
to create/modify, dependencies, acceptance test, and est (½–3 ED). Keep tasks small
enough to finish and verify in a single session.

## Delivery Cadence
- Sessions should complete **2–5 tasks** (one batch from the baton).
- Never start a new batch until the current Active tasks are done or explicitly
  deferred (with a reason in the baton).
- Quick Wins (high impact, low effort) are always prioritized within a batch.

