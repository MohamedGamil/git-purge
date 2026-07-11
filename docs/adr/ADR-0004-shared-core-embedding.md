# ADR-0004 — Shared core embedding: desktop runs without the CLI

| Field | Value |
| :--- | :--- |
| **Status** | Accepted |
| **Date** | 2026-07-11 |
| **Deciders** | Mohamed Gamil |
| **Relates to** | [02-architecture.md §4](../02-architecture.md), [ADR-0001](ADR-0001-hexagonal-architecture.md), [phase-4-ui.md](../../delivery/phase-4-ui.md) |

## Context

The user requirement states:

> "The UI tool should be able to run without the need of any browser as it will
> be a desktop application."

And more broadly, the CLI and UI should both use "abstractions of the same shared
codebase." This raises a question: does the desktop app **shell out** to the
`git-purge` CLI binary, or does it **embed** the core library?

## Decision

The desktop app **embeds `gitpurge-core` as a Rust library dependency** — it
never shells out to the CLI binary:

1. `gitpurge-desktop` (the Tauri backend crate) depends on `gitpurge-core` via
   `path = "../../../crates/gitpurge-core"` in `Cargo.toml`. It links the same
   compiled Rust code that the CLI links.

2. Every `#[tauri::command]` handler constructs or accesses a shared
   `Engine` instance (held in Tauri's managed state) and calls the same
   `Engine::scan`, `Engine::plan`, `Engine::execute`, etc. methods that the CLI
   calls.

3. The desktop app is **standalone-capable**: it works even if `git-purge` (the
   CLI binary) is not installed on the user's machine. There is no runtime
   dependency on the CLI.

4. The CLI binary `git-purge` is a separate binary. Both binaries are built from
   the same workspace and share the same `gitpurge-core` at compile time, but
   they are independently installable.

5. A `git-purge ui` subcommand in the CLI can **launch** the desktop app if it's
   installed, but this is a convenience, not a dependency. The desktop app never
   reverse-depends on the CLI.

## Consequences

- **Positive**: No IPC / subprocess coordination between CLI and UI. No "CLI not
  found" failure mode for desktop users. Identical behaviour guaranteed by the
  compiler (same library code). The desktop app has the same performance
  characteristics as the CLI.

- **Negative**: Two binaries in the release, each carrying a copy of the core
  library (though Rust's static linking means no runtime duplication). Updates
  must ship both if both are installed.

- **Risk**: Divergent configuration — if the CLI and desktop app resolve config
  from different paths, they could behave differently. Mitigated by using the
  same `Config::resolve()` logic (via `directories` crate) in both.

## Alternatives considered

| Alternative | Why rejected |
| :--- | :--- |
| Desktop shells out to CLI | Fragile: CLI must be installed, PATH must be correct, version must match. Parsing CLI stdout for structured data is error-prone. |
| CLI hosts a web server, UI connects | Over-engineering; requires port management; no benefit for a local single-user tool. |
| Single binary with `--gui` flag | Complicates packaging (must bundle webview assets in the CLI binary even for headless use); Tauri expects its own binary entry point. |
