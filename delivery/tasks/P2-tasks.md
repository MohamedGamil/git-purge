# P2 — Task Cards

> Phase: **2 — Backup & restore** · Status: **Not started** · Est: 10 ED
> Depends on: P1

---

## P2-T1 · Bare mirror init + object fetch

**Goal:** Create/open a bare mirror per tracked repo at `<data_dir>/gitpurge/backups/<repo-id>.git`.
Fetch objects from the source repo into the mirror.

**Files:** `crates/gitpurge-core/src/backup/mirror.rs`

**Depends on:** P1-T3

**Acceptance:** Mirror created on first backup; subsequent fetches add only new objects.

---

## P2-T2 · Snapshot create (namespaced refs)

**Goal:** Create a snapshot by writing namespaced refs
(`refs/gitpurge/backups/<snapshot-id>/<original-ref>`) into the bare mirror. Store
snapshot metadata as a JSON blob. Atomic: failed snapshot leaves no partial state.

**Files:** `crates/gitpurge-core/src/backup/snapshot.rs`

**Depends on:** P2-T1

**Acceptance:** Snapshot creates correct namespaced refs; metadata blob contains
timestamp, repo ID, ref list with OIDs.

---

## P2-T3 · Snapshot list/show/verify

**Goal:** Enumerate snapshots from namespaced refs; show details from metadata blob;
verify every referenced OID exists in the mirror's object database.

**Files:** `crates/gitpurge-core/src/backup/snapshot.rs`

**Depends on:** P2-T2

**Acceptance:** `verify` returns OK for valid snapshot; detects missing objects for
corrupted snapshot.

---

## P2-T4 · Restore (branch or tag)

**Goal:** Recreate a branch (or tag) in the source repo from a snapshot ref. Never
overwrites an existing ref without explicit consent (SAFE-06).

**Files:** `crates/gitpurge-core/src/action/restore.rs`

**Depends on:** P2-T3

**Acceptance:** Restore recreates deleted branch at correct OID; restore-as-tag works;
refusing overwrite leaves state unchanged; `SAFE-06` proven.

---

## P2-T5 · Snapshot prune + retention

**Goal:** Delete a snapshot's namespaced refs and run `git gc` on the mirror. Apply
retention policy (keep N, keep age).

**Files:** `crates/gitpurge-core/src/backup/prune.rs`

**Depends on:** P2-T3

**Acceptance:** Pruning removes only the target snapshot; objects shared with other
snapshots are retained.

---

## P2-T6 · Auto-restore on failed delete

**Goal:** When a delete operation fails mid-way, offer restore from the pre-op snapshot.
Declining is a no-op. Implements SAFE-05.

**Files:** `crates/gitpurge-core/src/action/mod.rs`

**Depends on:** P2-T4

**Acceptance:** Failed delete triggers restore offer; accepting restores; declining
leaves state unchanged; `SAFE-05` proven.

---

## P2-T7 · Engine wiring + round-trip & space tests

**Goal:** Wire `Engine::backup_create/list/show/verify/prune/restore`. Add the
snapshot→delete→restore round-trip and the sub-linear-space test.

**Files:** `crates/gitpurge-core/src/lib.rs`, `crates/gitpurge-core/src/backup/mod.rs`

**Depends on:** P2-T2 through P2-T6

**Acceptance:** Round-trip test restores byte-identically; 10 snapshots of an unchanged
repo grow the mirror ~O(refs), far below 10× repo size (ADR-0005 proof).
