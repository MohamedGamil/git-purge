# 01 — Tech Stack

`Status: Approved` · `Owner: Architecture` · `Last-updated: 2026-07-11` ·
`Related: 02-architecture.md, adr/`

This is the recommended stack **beyond Tauri**, chosen for the stated goals:
security, minimal maintenance, easy deployment, cross-platform reach, optimal
resource use, future-proofing, and developer ergonomics. Choices marked ✅ are
locked in [`../delivery/CONVENTIONS.md`](../delivery/CONVENTIONS.md).

## TL;DR

**One Rust core, two thin adapters, one webview UI.**

```
                ┌───────────────────────────────────────────────┐
                │   gitpurge-core  (Rust library, all logic)     │
                │   git engine · backup · policy · report · auth │
                └───────────────────────────────────────────────┘
                     ▲                              ▲
        depends on   │                              │  depends on
        ┌────────────┴─────────┐         ┌──────────┴───────────────┐
        │  gitpurge-cli (bin)  │         │  gitpurge-desktop (Tauri) │
        │  clap + miette       │         │  Rust cmds ⇄ Vue 3 webview │
        └──────────────────────┘         └───────────────────────────┘
```

Because both the CLI and the Tauri backend are Rust and both link the **same
library crate**, "CLI and UI share the same logic" is guaranteed by the compiler,
not by discipline. The UI needs no separate CLI process and no browser.

## Why Rust for the core

| Goal | How Rust delivers |
| :--- | :--- |
| Security | Memory safety, no data races, `#![forbid(unsafe_code)]` in core; typed errors; no shell-injection surface (we call libraries, not `sh -c`). |
| Single self-contained binary | Static-ish binaries with no runtime/interpreter. The old scripts needed bash + python3 + git; Git Purge needs nothing installed. |
| Tauri alignment | Tauri's backend **is** Rust — the core links directly into the desktop app. No FFI, no IPC to a sidecar, no duplicated logic. |
| Performance / memory | Handles repos with thousands of refs (the source repos had 2,300+ branches) with low RAM and fast traversal. |
| Cross-platform | First-class Linux/macOS/Windows; cross-compiles cleanly. |
| Maintenance | Strong types + exhaustive matching + a large test net reduce regressions; one language across core/CLI/desktop-backend. |

## Component choices

### Git engine — hybrid `gix` + `git2` (✅ ADR-0002)
- **`gix` (gitoxide)** — pure-Rust, fast, no C dep → best for static single-binary
  builds and read-heavy work (ref listing, commit walk, diff, ancestry).
- **`git2` (libgit2)** — mature completeness for push/delete/fetch + credential
  callbacks where gix is still maturing.
- Both sit behind a `GitBackend` trait; a `ShellGitBackend` (system `git`) exists
  only as a diagnostic/parity fallback. See [04-core-spec.md](04-core-spec.md).

### CLI
| Concern | Crate | Why |
| :--- | :--- | :--- |
| Arg parsing | `clap` v4 (derive) | Standard, generates help + completions. |
| Error UX | `miette` + `thiserror` | Rich diagnostics with hints; typed errors. |
| Color/style | `anstyle` / `owo-colors` | Cross-platform ANSI, respects `NO_COLOR`. |
| Tables/output | `comfy-table` + `serde_json` | Human tables and `--json` for scripting. |
| Progress | `indicatif` | Progress bars for long ops. |
| Prompts | `dialoguer` | Confirmations (respects `--yes`). |

### Desktop (Tauri v2)
| Concern | Choice | Why |
| :--- | :--- | :--- |
| Shell | **Tauri v2** | Tiny bundles, OS webview (no bundled Chromium), Rust backend. |
| UI framework | **Vue 3 + Vite + TypeScript** (✅) | Matches the team's existing frontend; reactive, small runtime, great DX. |
| State | **Pinia** | Simple, typed stores. |
| Routing | **Vue Router 4** | Screen navigation. |
| Styling | Design tokens (CSS custom properties) implementing **One Dark Pro + Material** — see [07-ui-design-system.md](07-ui-design-system.md). Lightweight, no heavy component lib. |
| Data viz | `d3`-lite / lightweight SVG for branch graph & trend charts | Avoids heavyweight charting deps. |
| IPC | Tauri commands + events | `invoke()` for calls, event stream for progress. |

### Persistence & platform
| Concern | Crate | Why |
| :--- | :--- | :--- |
| History/trends DB | `rusqlite` (bundled SQLite) | Robust queries/trends vs. the old flat JSON; no server. |
| Config | `toml` + `serde` | Human-editable config. |
| Paths | `directories` | Correct per-OS config/data/cache dirs (no hardcoded `/home/...`). |
| Secrets | `keyring` (OS keychain) + encrypted-file fallback (`age`/`aes-gcm`) | See [09-authentication.md](09-authentication.md). |
| Async | `tokio` | Concurrency for network/git ops; cancellation. |
| Logging | `tracing` + `tracing-subscriber` | Structured logs / JSON lines. |
| Serialization | `serde` | Everywhere. |

### Quality & supply chain
`cargo fmt`, `cargo clippy -D warnings`, `cargo nextest`, `cargo-deny`
(license/advisory/ban checks), `insta` (snapshots), `proptest` (property tests for
classification/policy), `cargo-dist` or handwritten `release.yml` for artifacts.

## Alternatives considered (and why not)

| Decision | Alternative | Why not chosen |
| :--- | :--- | :--- |
| Core language | Go | Good single-binary story, but Tauri backend is Rust → would force a second language + FFI/IPC and split logic. Fails Requirement 6 cleanly. |
| Core language | Node/TS (shared with UI) | Heavy runtime, weaker for a self-contained CLI binary, slower git traversal at 2k+ refs. |
| Git engine | libgit2 only | Simple but adds a C dep everywhere and complicates fully-static builds. Hybrid keeps reads pure-Rust. |
| Git engine | shell out to `git` | Breaks "zero-setup single binary" (requires git installed) and re-introduces shell-injection risk. Kept only as fallback. |
| UI | Svelte/Solid/React | All viable; Vue 3 chosen to match the team's existing stack and reduce ramp-up. |
| DB | Flat JSON (as today) | Doesn't scale to rich trend queries; fragile concurrent writes. SQLite is embedded and robust. |

## Consequences
- Contributors need the Rust toolchain + Node/pnpm (for the desktop frontend only).
- The CLI has **no** runtime dependencies; the desktop app depends only on the OS
  webview (WebView2 on Windows — bootstrapped by the installer; WebKitGTK on Linux).
- All logic is testable headlessly in `gitpurge-core` without spawning the UI.
