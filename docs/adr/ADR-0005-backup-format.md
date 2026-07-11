# ADR-0005 — Backup format: namespaced refs in a shared bare mirror

| Field | Value |
| :--- | :--- |
| **Status** | Accepted |
| **Date** | 2026-07-11 |
| **Deciders** | Mohamed Gamil |
| **Relates to** | [08-backup-and-restore.md §3](../08-backup-and-restore.md), [CONVENTIONS §6](../../delivery/CONVENTIONS.md), [03-domain-model.md §6](../03-domain-model.md), [phase-2-backup-restore.md](../../delivery/phase-2-backup-restore.md) |

## Context

Git Purge's safety model (SAFE-04) requires that every destructive operation is
preceded by a verified backup. Users may run cleanup repeatedly (weekly, monthly)
and accumulate many snapshots. The backup format must satisfy:

1. **Space efficiency**: N snapshots of the same repo should cost much less than
   N × repo size.
2. **Atomic**: a failed snapshot must not leave partial state.
3. **Restorable**: any individual branch can be restored from any snapshot.
4. **Verifiable**: snapshot integrity can be checked without restoring.
5. **Prunable**: old snapshots can be removed without affecting newer ones.
6. **Portable**: works on any platform, no platform-specific tooling.

## Decision

Use **namespaced refs inside a shared bare git mirror** as the backup format:

1. **One bare mirror per tracked repository**, stored at a well-known location
   inside the Git Purge data directory:
   ```
   $XDG_DATA_HOME/gitpurge/backups/<repo-id>.git   (bare)
   ```

2. **Each snapshot is a set of namespaced refs**:
   ```
   refs/gitpurge/backups/<snapshot-id>/<original-ref-name>
   ```
   For example, backing up `refs/heads/feature/login` in snapshot `20260711T010000Z`
   creates `refs/gitpurge/backups/20260711T010000Z/refs/heads/feature/login`
   pointing at the same OID.

3. **Objects are shared** — the bare mirror's object database holds objects from
   all snapshots. N snapshots of an unchanged repo add only N × (number of refs)
   new ref entries, not N × (object count). This is the **"minimal space" rule**
   from CONVENTIONS §6.

4. **Snapshot metadata** is stored as a blob at
   `refs/gitpurge/backups/<snapshot-id>/.metadata` containing a JSON manifest:
   timestamp, repo ID, ref list with OIDs, and the operation that triggered the
   snapshot.

5. **Verification** checks that every OID referenced by a snapshot's refs exists
   in the mirror's object database (`git fsck`-style).

6. **Pruning** deletes a snapshot's namespaced refs and runs `git gc` on the
   mirror. Objects still referenced by other snapshots are retained.

7. **Restore** creates a branch (or tag) in the original repo pointing at the
   OID from the snapshot ref. Never overwrites an existing ref without explicit
   consent (SAFE-06).

## Space analysis

For a repo with B branches and N snapshots where the repo doesn't change between
snapshots:

- **This approach**: N × B ref entries + 1 copy of the objects ≈ O(B × N) bytes
  for refs + O(objects) for the pack.
- **Naive approach** (N full copies): N × O(repo size). For a 100 MB repo with
  10 snapshots, that's ~1 GB vs. ~100 MB + negligible ref overhead.

When the repo *does* change between snapshots, only new objects are added to the
shared pack. Git's delta compression further reduces the cost.

## Consequences

- **Positive**: Near-zero marginal cost for incremental snapshots. Uses standard
  git primitives (refs, objects, packs) — no custom binary format. Portable
  across platforms. Verification is just `git fsck`.

- **Negative**: Requires a bare repo per tracked repository. Pruning requires
  garbage collection. The backup directory can grow if many objects are unique
  across snapshots (but this is inherent to any backup system).

- **Risk**: Very large repos (>10 GB object DB) may have slow initial backup.
  Mitigated by documenting that the first backup may take time, and subsequent
  ones are incremental.

## Alternatives considered

| Alternative | Why rejected |
| :--- | :--- |
| Git bundles (one `.bundle` per snapshot) | Each bundle is a full copy — fails the space-efficiency requirement. |
| Tar/zip of `.git` directory | Same space problem; plus, not random-access (can't restore a single branch without extracting everything). |
| Custom binary format | Reinventing git's object storage; maintenance burden; no ecosystem tooling for verification. |
| SQLite blob store | Would need to re-implement object deduplication and delta compression that git already does. |
| `git stash`-like approach | Stashes are local-only, hard to enumerate, and don't support named snapshots with metadata. |
