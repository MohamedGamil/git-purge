# ADR-0002 — Hybrid git engine: gix (primary) + git2 (fallback)

| Field | Value |
| :--- | :--- |
| **Status** | Amended |
| **Date** | 2026-07-11 |
| **Deciders** | Mohamed Gamil |
| **Relates to** | [01-tech-stack.md §git-engine](../01-tech-stack.md), [04-core-spec.md §4](../04-core-spec.md), [CONVENTIONS §4](../../delivery/CONVENTIONS.md) |

## Context

Git Purge needs to enumerate branches, resolve ancestry, compute merge-base,
delete refs (local and remote), push deletions, and create/verify backup
snapshots. The Rust ecosystem has two main git libraries:

| Library | Strengths | Weaknesses |
| :--- | :--- | :--- |
| **gix (gitoxide)** | Pure Rust, no C deps, excellent read perf, active development | Write / push / credential-callback support still maturing |
| **git2 (libgit2)** | Mature, full write support, well-tested credential callbacks | C dependency (OpenSSL, libssh2), complicates musl / cross builds |

Neither library covers 100 % of our needs alone. A pure-gix approach hits gaps
on push and credential callbacks; a pure-git2 approach brings C build complexity
and loses the performance and safety benefits of gitoxide.

## Decision

Use a **hybrid strategy** behind the `GitBackend` port trait (ADR-0001):

1. **gix is the primary backend** for all read operations: ref enumeration,
   object lookup, tree/blob reads, ancestry checks, merge-base computation, diff
   stats.

2. **git2 is the fallback** for operations gix doesn't yet cover well: remote
   push (for `--delete` on remotes), credential callbacks (SSH agent, HTTPS
   token prompting), and any write path where gix's API is unstable.

3. A **composite `GitBackend`** implementation routes each method call to gix or
   git2 based on a capability table. The routing is internal to the adapter
   crate — `gitpurge-core` sees only the `GitBackend` trait.

4. **System `git` shell-out** fallback is dropped and not supported. Native Rust libraries
   (`gix` and `git2`) completely cover the engine's requirements without requiring
   external Git CLI dependencies. This satisfies the single-binary, zero-setup design goal.

5. Priority order: **gix → git2**. As gix matures, operations migrate
   from git2 to gix without any facade change (the trait surface stays stable).

## Consequences

- **Positive**: Best-of-both-worlds — pure-Rust performance for reads, proven
  reliability for writes. The `GitBackend` trait means the routing is invisible
  to the rest of the codebase.

- **Negative**: Two git libraries in the dependency tree; slightly larger binary
  size; must keep routing table up to date as gix adds features.

- **Risk**: gix API instability between releases. Mitigated by pinning gix
  versions and testing against the fixture repos (testkit).

## Alternatives considered

| Alternative | Why rejected |
| :--- | :--- |
| gix-only | Push / credential support not mature enough for production use today. |
| git2-only | C deps complicate cross-compilation; loses pure-Rust benefits. |
| Shell-out only | Fragile parsing; security risk from command injection; no in-process object access. |
| JGit / go-git via FFI | Wrong ecosystem; FFI overhead and complexity for a Rust project. |
