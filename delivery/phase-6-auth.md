# Phase 6 — Authentication & secure storage

`Status: Complete` · `Owner: Delivery` · `Last-updated: 2026-07-17` ·
`Related: [ROADMAP](../docs/ROADMAP.md#p6--authentication--secure-storage-9-ed), [CONVENTIONS §4/§5](CONVENTIONS.md), [architecture §3](../docs/02-architecture.md), [09-authentication.md](../docs/09-authentication.md), [14-security.md](../docs/14-security.md)`

## Goal

Let Git Purge authenticate to real remotes without ever leaking a secret
(Requirement 5). Implement the `SecretStore` adapters — OS keychain via `keyring`
with an encrypted-file fallback (`age`/`aes-gcm`) — a credential model covering SSH
keys, HTTPS user/pass, and tokens, and a resolver that falls back to the system
SSH-agent/identity or user-provided keys/identities. Bridge resolved credentials into
the `git2` fetch/push callbacks, and expose `auth add|list|remove|test`. Secrets never
touch logs, errors, snapshots, or reports (SAFE-07). This phase parallelizes with
P3/P4/P5 (depends only on P1).

**Milestone:** M3 — CLI 1.0 (auth completes the CLI surface).

**Dependencies:** **P1** merged (the `git2` backend that credential callbacks attach
to; the `SecretStore` port from P0). Consumed by **P3** (`auth` verb) and **P4**
(auth manager), and unlocks the live-remote paths in P2/P3.

## Tasks

| Task ID | Title | Description | Files (repo-relative) | Depends-on | ∥? | Est (ED) | Acceptance test |
| :-- | :-- | :-- | :-- | :-- | :-- | :-- | :-- |
| **P6-T1** | Keyring `SecretStore` adapter | Implement the `SecretStore` port over the OS keychain via `keyring` (secrets keyed by `RepoId`/host + credential id). `keyring` stays out of CLI/UI crates ([architecture §1](../docs/02-architecture.md)). | `crates/gitpurge-core/src/auth/keyring_store.rs` | P0-T4 | yes | 1.5 | Store→read→delete a secret via the keyring adapter (using the test keychain / fake in CI); secret value never appears in `Debug`/logs (**SAFE-07**); **R5**. |
| **P6-T2** | Encrypted-file fallback store | Fallback `SecretStore` writing an encrypted file (`age`/`aes-gcm`) under the `directories` data dir when no OS keychain is available; key derivation + at-rest encryption; automatic selection when keyring is absent. | `crates/gitpurge-core/src/auth/file_store.rs` | P6-T1 | yes | 1.5 | On a keyring-less environment the fallback stores/reads a secret; the on-disk file is ciphertext (plaintext secret never present); **SAFE-07**, **R5**. |
| **P6-T3** | Credential model + providers | Types for credential kinds — SSH key(path/passphrase), HTTPS user/pass, token — with (de)serialization that keeps secret material in the store, not in config. `Debug`/serialize redacts secrets. | `crates/gitpurge-core/src/auth/{credential.rs,provider.rs}`, `crates/gitpurge-core/src/model/auth.rs` | P0-T4 | yes | 1.5 | Each credential kind constructs and persists a reference (secret in `SecretStore`, metadata in config); `format!("{:?}")` shows `<redacted>` for secret fields (**SAFE-07**). |
| **P6-T4** | Credential resolver + system fallback | Resolve the credential for a given remote in order: explicit configured credential → user-provided key/identity → **system SSH-agent/identity** fallback. Deterministic, logged (without secrets) for auditability. | `crates/gitpurge-core/src/auth/resolver.rs` | P6-T3 | no | 1.5 | Resolver picks the configured credential when present, else falls back to the system SSH identity path; the fallback branch is exercised by a test; **R5**. |
| **P6-T5** | git2 credential-callback bridge | Wire the resolver into `git2`'s fetch/push credential callbacks so authenticated fetch/push/delete work through `GitBackend`; retry/renegotiate on auth failure without echoing secrets. | `crates/gitpurge-core/src/git/git2_backend.rs`, `crates/gitpurge-core/src/auth/git_bridge.rs` | P6-T4, P1-T4 | no | 1.5 | Authenticated fetch/push succeeds against an HTTPS-token remote and an SSH remote (local test servers / fixtures); a failed auth produces a typed error with **no secret** in the message (**SAFE-07**); **R5**. |
| **P6-T6** | `Engine::auth_*` + CLI `auth` verb | Wire `Engine::auth_add/list/remove/test`; back the P3 `auth add|list|remove|test` verb (thin). `auth test` performs a real credential handshake and reports success/failure without revealing the secret. | `crates/gitpurge-core/src/auth/mod.rs`, `crates/gitpurge-core/src/lib.rs`, `crates/gitpurge-cli/src/cmd/auth.rs` | P6-T4, P6-T5 | no | 1 | `git-purge auth add`/`list`/`remove` manage credentials; `auth test` reports reachable/unreachable; `list` output never prints secrets; **R5**. |
| **P6-T7** | Secret-hygiene regression suite | Named regression tests asserting no secret material appears in logs, errors, snapshots, or reports across the auth paths; keychain fake used so tests are network-free. | `crates/gitpurge-core/tests/secret_hygiene.rs` | P6-T1, P6-T2, P6-T3, P6-T6 | yes | 0.5 | `safe_07` test scans captured logs/errors/serialized snapshots+reports for known secret sentinels and asserts absence; **SAFE-07**. |

Total ≈ 9 ED.

## Exit criteria

- Authenticated fetch/push against HTTPS-token and SSH remotes works, and secrets
  never touch logs or plaintext (ROADMAP P6 exit).
- `SecretStore` has both keychain and encrypted-file adapters, selected automatically;
  the resolver falls back to the system SSH identity.
- `auth add|list|remove|test` is functional through `Engine`; the CLI/UI never render
  secrets.

### Requirements & safety invariants satisfied

- **R5** (multiple auth methods, secure storage, system + user-provided fallbacks):
  P6-T1..T6.
- **R6** (`keyring`/crypto behind a port; not in CLI/UI): P6-T1, P6-T2 (arch ban).
- **SAFE-07** (no secret material in logs/errors/snapshots/reports): pervasive; proven
  by P6-T7.

## Risks & open questions

- **Testing real auth without network** — stand up local HTTPS/SSH git servers or use
  `git2` against `file://` + loopback for CI; keep the keychain a fake so tests are
  deterministic (no touching the developer's real keychain).
- **Keyring availability** — headless Linux CI often lacks a Secret Service; the
  encrypted-file fallback must kick in automatically and be the CI default.
- **Encrypted-file key management** — where the file-store master key lives (OS keychain
  entry vs. passphrase prompt) needs a decision in [09-authentication.md](../docs/09-authentication.md);
  default to a keychain-held key with a passphrase fallback.
- **SSH-agent/identity discovery** — cross-platform agent socket / identity file
  discovery differs by OS; document the resolution order and gate live tests per OS.
- **gix vs git2 for authed transport** — auth callbacks live in git2 today (ADR-0002);
  revisit if/when gix's authenticated transport matures.
