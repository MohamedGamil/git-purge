# Delivery

This directory turns the [specs](../docs/) into **agent-runnable work**. It is the
operational plan for building Git Purge across many focused engineer/agent sessions.

## Contents
- [`CONVENTIONS.md`](CONVENTIONS.md) — **canonical decisions. Source of truth. Read first.**
- [`AGENT_GUIDE.md`](AGENT_GUIDE.md) — how an implementation agent should work: setup,
  workflow, guardrails, how to pick up a task, how to report done.
- [`DEFINITION_OF_DONE.md`](DEFINITION_OF_DONE.md) — global DoD + R1–R12 traceability matrix.
- `phase-0-foundations.md` … `phase-8-hardening-release.md` — one doc per roadmap
  phase, each a checklist of tasks with IDs, acceptance criteria, and file targets.
- [`tasks/`](tasks/) — optional expanded task cards for large/ambiguous tasks.

## How to use (for a human orchestrator or a lead agent)
1. Read [`CONVENTIONS.md`](CONVENTIONS.md) and the relevant [`../docs/`](../docs/) spec.
2. Pick the lowest-numbered phase whose dependencies are met (see
   [ROADMAP](../docs/ROADMAP.md#dependency-graph)).
3. Assign its tasks to agents. Tasks within a phase marked `∥` can run in parallel.
4. Each agent follows [`AGENT_GUIDE.md`](AGENT_GUIDE.md) and closes tasks against the
   [DoD](DEFINITION_OF_DONE.md).
5. A phase is complete only when its **Exit criteria** (in the phase doc) and all
   mapped requirements pass tests.

## Task ID scheme
`P<phase>-T<n>` (e.g. `P1-T3`). Sub-tasks: `P1-T3.a`. Each task lists: goal, files
to create/modify, dependencies, acceptance test, and est (½–3 ED). Keep tasks small
enough to finish and verify in a single session.
