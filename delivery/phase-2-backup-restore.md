# Phase 2 — Backup & restore

`Status: Draft` · `Owner: Delivery` · `Last-updated: 2026-07-11` ·
`Related: [ROADMAP](../docs/ROADMAP.md#p2--backup--restore-10-ed), [CONVENTIONS §6/§7](CONVENTIONS.md), [domain model §6](../docs/03-domain-model.md), [ADR-0005](../docs/adr/ADR-0005-backup-format.md), [08-backup-and-restore.md](../docs/08-backup-and-restore.md), [11-safety-model.md](../docs/11-safety-model.md)`

## Goal

Implement the safety net every destructive operation depends on: the
minimal-space backup model (ADR-0005) and restore. One **bare mirror per repo**
holds all snapshots; each snapshot writes captured refs into a namespaced backup
ref (`refs/gitpurge/backups/<snapshot-id>/<original-ref-path>`) plus a
`snapshot.json` manifest, so N snapshots cost ~O(changed objects), not N clones.
Add snapshot `create/list/show/verify/prune`, restore-as-branch and restore-as-tag
with consent (never force-overwrite), and the auto-restore-on-failed-delete wrapper
that later actions run through. After P2 the tool can safely capture and reverse
state, satisfying Requirement 2/10.

**Milestone:** M2 — Safe mutation (backup half; delete/archive land in P3).

**Dependencies:** **P1** merged (`GitBackend`, refs/commits, `model::Snapshot`
shape). P0 `HistoryStore` port is present; its SQLite adapter arrives in P5, so P2
persists snapshot metadata through the port (fake in tests, real store later).

## Tasks

| Task ID | Title | Description | Files (repo-relative) | Depends-on | ∥? | Est (ED) | Acceptance test |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| **P2-T1** | Bare-mirror-per-repo manager | Create/open `backups/<repo-id>.git` (a bare mirror) under the `directories`-resolved backups root; fetch/update all source refs into the mirror. Backups root overridable per repo + global ([CONVENTIONS §5/§6](CONVENTIONS.md)). Generalizes `backup_repos.sh` (managed, space-shared). | `crates/gitpurge-core/src/backup/mirror.rs` | P1-T1 | no | 1.5 | Mirroring `multi_remote_repo` creates one bare repo containing all source objects/refs; re-mirroring reuses the same bare repo (no second clone); **R2/R10**. |
| **P2-T2** | Snapshot create (namespaced refs + manifest) | Write each captured ref to `refs/gitpurge/backups/<snapshot-id>/<orig>` inside the mirror and a `snapshot.json` manifest; build `Snapshot`/`SnapshotRef` metadata (id ULID, timestamp, trigger, branch, tip SHA, `commit_count`, upstream, `merged_at_capture`). No new clone. | `crates/gitpurge-core/src/backup/snapshot.rs`, `crates/gitpurge-core/src/model/snapshot.rs` | P2-T1 | no | 2 | Snapshot of a fixture writes backup refs + manifest; `SnapshotRef.commit_count` equals reachable commits (proves "content backed up"); **R2**, **SAFE-04** groundwork. |
| **P2-T3** | Snapshot verify | Read every captured object back from the mirror and confirm reachability; set `Snapshot.verified`. This is the "verified" in backup-before-destroy. | `crates/gitpurge-core/src/backup/verify.rs` | P2-T2 | no | 1 | `verify` on a good snapshot → `verified=true`; corrupting/removing a backup ref → `verify` fails and `verified=false`; **SAFE-04**. |
| **P2-T4** | list / show / prune | Enumerate snapshots (from manifest + `HistoryStore` port), `show` one snapshot's refs/metadata, and `prune` by age/count/keep-last policy — pruning removes backup refs and lets git GC reclaim now-unreferenced objects. | `crates/gitpurge-core/src/backup/{list.rs,show.rs,prune.rs}` | P2-T2 | yes | 1 | After 3 snapshots, `list` returns 3 sorted by time; `prune --keep-last 1` leaves 1 and drops the other two backup refs; `show` renders one snapshot's captured refs. |
| **P2-T5** | Restore as branch / as tag (consent) | Recreate a captured ref as a **branch or a tag** (`RestoreSpec.as_kind`), honoring `OnConflict` (`Abort` default / `RestoreUnderNewName` / `Overwrite` only with explicit consent token). Never force-overwrite silently. Generalizes `restore_repos.sh`. | `crates/gitpurge-core/src/backup/restore.rs`, `crates/gitpurge-core/src/action/restore.rs` | P2-T2 | yes | 2 | Restore-as-branch recreates the branch at the captured SHA (byte-identical tip); restore-as-tag creates a tag; restoring onto an existing ref **aborts** unless consent given — **R2**, **SAFE-06**. |
| **P2-T6** | Auto-restore-on-failed-delete wrapper | A guarded-execution wrapper: before a destructive op it ensures a verified pre-op snapshot exists; on op failure it **offers** restore from that snapshot; declining is a no-op. This is the seam `execute` (P3) runs every destructive item through. | `crates/gitpurge-core/src/action/guard.rs` | P2-T2, P2-T5 | no | 1.5 | Inject a `GitBackend` fake whose delete fails: wrapper offers restore and, on accept, the branch returns identical; on decline, state is unchanged (no partial mutation) — **SAFE-05**. |
| **P2-T7** | Engine wiring + round-trip & space tests | Wire `Engine::backup_create/backup_list/backup_show/backup_verify/backup_prune/restore`; add the snapshot→delete→verify-gone→restore round-trip and the sub-linear-space test. | `crates/gitpurge-core/src/lib.rs`, `crates/gitpurge-core/src/backup/mod.rs` | P2-T3, P2-T4, P2-T5, P2-T6 | no | 1 | Round-trip test restores byte-identically; disk-size test: 10 snapshots of an unchanged repo grow the mirror ~O(refs), far below 10× repo size — proves the ADR-0005 minimal-space claim. |

Total ≈ 10 ED.

## Exit criteria

- Snapshot → delete → verify-gone → restore round-trips **byte-identically**, and
  space stays sub-linear across snapshots (ROADMAP P2 exit).
- A snapshot is verifiable, and restore can materialize a branch *or* a tag with
  consent, never force-overwriting.
- The auto-restore-on-failed-delete wrapper exists and is proven by a
  fault-injection test; P3's `execute` will route every destructive item through it.

### Requirements & safety invariants satisfied

- **R2** (backup before delete; restore; auto-restore; branch/tag; no force): P2-T2,
  P2-T5, P2-T6.
- **R10** (portable restore semantics): P2-T1, P2-T5.
- **SAFE-04** (verified pre-op snapshot before destroy): P2-T2, P2-T3, P2-T6.
- **SAFE-05** (failed delete offers restore; decline is no-op): P2-T6.
- **SAFE-06** (restore never force-overwrites without consent): P2-T5.
- **SAFE-03** (tags never deleted by branch ops) reinforced: restore *creates* tags;
  the branch-op tag guard is finalized in P3's action layer.

## Risks & open questions

- **gix vs git2 for ref writes into the mirror** — writing namespaced refs and
  fetching into a bare repo may be more robust via git2; keep it behind `GitBackend`
  and let the composite decide (ADR-0002).
- **Prune vs. GC timing** — dropping a backup ref does not immediately reclaim space;
  decide whether `prune` triggers a repack/`gc` or leaves it to a maintenance task
  (the space test must account for this).
- **repo-id ⇄ mirror path collisions** — `RepoId` includes a path hash; confirm two
  distinct working copies of the same remote map to distinct mirrors as intended.
- **HistoryStore not yet real (P5)** — snapshot metadata is persisted through the port
  with a fake in P2; ensure the manifest (`snapshot.json`) is the source of truth so
  restore works even before the SQLite store exists.
- **Very large repos** — mirror + verify cost on 2k+ ref repos is measured in P8; P2
  must not assume small repos.
