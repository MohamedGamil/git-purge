# ADR-0003 — Desktop UI: Tauri v2 + Vue 3

| Field | Value |
| :--- | :--- |
| **Status** | Accepted |
| **Date** | 2026-07-11 |
| **Deciders** | Mohamed Gamil |
| **Relates to** | [01-tech-stack.md](../01-tech-stack.md), [06-ui-spec.md](../06-ui-spec.md), [07-ui-design-system.md](../07-ui-design-system.md), [CONVENTIONS §10](../../delivery/CONVENTIONS.md), [phase-4-ui.md](../../delivery/phase-4-ui.md) |

## Context

The user requirement specifies a desktop application that:

- Runs **without any browser** — a standalone desktop app, not an Electron web app.
- Ships as a **portable package** (`.deb`, `.rpm`, `.AppImage`, `.msi`, `.dmg`).
- Uses the **same Rust core** as the CLI (ADR-0001, ADR-0004).
- Has a **minimalistic, intuitive UI** with Material-inspired design using the
  One Dark Pro colour palette, supporting light/dark/system theme modes.

We need a framework that embeds a webview (for rich UI) but ships as a native
binary without bundling a full browser engine.

## Decision

Use **Tauri v2** for the desktop shell and **Vue 3** (Composition API + Vite +
TypeScript) for the frontend:

1. **Tauri v2** provides a native window with the system webview (WebKitGTK on
   Linux, WebView2 on Windows, WKWebView on macOS). No bundled Chromium — binary
   size stays small (~10 MB vs ~150 MB for Electron).

2. **Vue 3** with Composition API + `<script setup>` for the UI layer. Pinia for
   state management. Vite as the build tool. TypeScript throughout.

3. **IPC model**: Vue calls `@tauri-apps/api/core.invoke(commandName, args)` →
   Tauri routes to `#[tauri::command]` Rust functions → Rust functions call
   `gitpurge-core::Engine` methods → results return as JSON.

4. **Events**: Tauri event system for progress updates, theme changes, and
   async notifications (e.g., scan complete while user is on another view).

5. **Design system**: Custom CSS tokens based on One Dark Pro + Material Design
   principles (see `07-ui-design-system.md`). No heavy component library — the
   app is simple enough that a bespoke design system is cleaner.

## Consequences

- **Positive**: Small binary; native feel; system webview means no security
  surface from bundled Chromium; Rust backend is the same code as the CLI.

- **Negative**: System webview version varies across OS/distro — must test
  across WebKitGTK versions. Some CSS features (e.g., backdrop-filter) behave
  differently across webview engines.

- **Risk**: WebKitGTK on older Linux distros may lack features. Mitigated by
  targeting WebKitGTK 4.1+ (Ubuntu 22.04+) and documenting minimum requirements.

## Alternatives considered

| Alternative | Why rejected |
| :--- | :--- |
| Electron | Bundles Chromium (~150 MB); different runtime model; doesn't align with "portable, lightweight" goal. |
| GTK4 / FLTK / egui (native Rust UI) | Much harder to build the rich, themed UI the user wants; limited ecosystem for Material-style components. |
| React / Svelte (instead of Vue) | All viable; Vue chosen per user preference and ecosystem familiarity. |
| Tauri v1 | v2 has better multi-window, permissions model, and plugin system; v1 is now legacy. |
