# Architecture Decision Records

This directory contains Architecture Decision Records (ADRs) for Git Purge.
ADRs document significant architectural and design decisions with their context,
rationale, and consequences.

## Format

Each ADR follows a consistent structure:
- **Context**: The problem or situation that prompted the decision.
- **Decision**: What we decided to do and why.
- **Consequences**: Positive, negative, and risk implications.
- **Alternatives considered**: What else we evaluated and why it was rejected.

## Index

| ADR | Title | Status |
| :--- | :--- | :--- |
| [ADR-0001](ADR-0001-hexagonal-architecture.md) | Hexagonal architecture with ports & adapters | Accepted |
| [ADR-0002](ADR-0002-git-engine-hybrid.md) | Hybrid git engine: gix (primary) + git2 (fallback) | Accepted |
| [ADR-0003](ADR-0003-ui-vue-tauri.md) | Desktop UI: Tauri v2 + Vue 3 | Accepted |
| [ADR-0004](ADR-0004-shared-core-embedding.md) | Shared core embedding: desktop runs without CLI | Accepted |
| [ADR-0005](ADR-0005-backup-format.md) | Backup format: namespaced refs in shared bare mirror | Accepted |

## Adding a new ADR

1. Create `ADR-NNNN-<short-slug>.md` using the template above.
2. Add it to the index in this README.
3. Set status to `Proposed` until reviewed, then `Accepted` or `Rejected`.
4. Cross-reference from the relevant spec/delivery docs.

Canonical decisions live in [`../delivery/CONVENTIONS.md`](../../delivery/CONVENTIONS.md).
If an ADR changes a convention, update CONVENTIONS.md to match.
