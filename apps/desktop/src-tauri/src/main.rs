//! `gitpurge-desktop` — Tauri v2 backend (CONVENTIONS §2).
//!
//! This is a thin adapter that exposes `gitpurge-core::Engine` methods as Tauri
//! commands for the Vue 3 frontend. **No git/DB/keychain logic lives here.**
//!
//! ## Note (P0 scaffolding)
//! This is a minimal stub that compiles but does nothing. The full Tauri app
//! (commands, events, window setup) lands in Phase 4.

// Prevent a console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // TODO(P4): initialize Tauri app builder with commands and event handlers.
    // For P0 scaffolding, this is a no-op binary that proves the crate compiles
    // as a workspace member.
    println!("gitpurge-desktop: Tauri app stub — full UI lands in Phase 4");
}
