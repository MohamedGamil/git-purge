# ADR-0006 — Libgit2 global timeouts configuration with unsafe block

| Field | Value |
| :--- | :--- |
| **Status** | Accepted |
| **Date** | 2026-07-12 |
| **Deciders** | Mohamed Gamil |
| **Relates to** | [01-tech-stack.md](../01-tech-stack.md), [CONVENTIONS §11](../../delivery/CONVENTIONS.md), [lib.rs](../../crates/gitpurge-core/src/lib.rs) |

## Context

In network environments with Git servers behind corporate VPNs or restricted networks, losing connection to these servers can cause libgit2 connection attempts to block indefinitely or hit extremely long system-level TCP timeouts (which can exceed 75 seconds). During this time, the application appears unresponsive.

We need to enforce connection and operation timeouts on all Git remote operations. Because libgit2 manages connection states globally, these timeouts must be configured globally using `git2::opts::set_server_connect_timeout_in_milliseconds` and `git2::opts::set_server_timeout_in_milliseconds`.

However, the `git2-rs` crate defines these functions as `unsafe fn` because they mutate global library state. `gitpurge-core` uses a `#![forbid(unsafe_code)]` compiler directive to ensure memory safety.

## Decision

Exempt this specific global configuration from the `unsafe` ban:

1. Change `gitpurge-core`'s top-level compiler directive from `#![forbid(unsafe_code)]` to `#![deny(unsafe_code)]` to allow local selective exemptions while keeping the safety denial active for all other code.
2. In `Engine::open` inside `crates/gitpurge-core/src/lib.rs`, wrap the global libgit2 option calls in `#[allow(unsafe_code)] unsafe { ... }`.
3. Set the connection timeout to 5 seconds (`5000` ms) and the operation timeout to 15 seconds (`15000` ms) during engine initialization.

## Consequences

- **Positive**: Prevents the application from freezing or hanging when remote Git servers are unreachable due to VPN loss or lack of network connectivity. Network actions fail gracefully within 5 seconds of connection attempt or 15 seconds of operation.
- **Negative**: Relies on a single unsafe block in `gitpurge-core`.
- **Mitigation**: The unsafe block only calls static libgit2 configuration functions during engine initialization, which modifies static primitive integer parameters and poses zero risk of memory corruption or safety violations.

## Alternatives considered

| Alternative | Why rejected |
| :--- | :--- |
| Custom SSH/HTTP transport | Extremely complex to implement safely, introduces a massive unsafe FFI surface. |
| External process spawning | Slower, bypasses the hybrid gix/git2 engine architecture, and introduces shell injection risks. |
