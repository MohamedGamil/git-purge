//! `gitpurge-desktop` — Tauri v2 backend (CONVENTIONS §2).
//!
//! This is a thin adapter that exposes `gitpurge-core::Engine` methods as Tauri
//! commands for the Vue 3 frontend. **No git/DB/keychain logic lives here.**

// Prevent a console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

pub mod commands;

/// Managed state for the Tauri application.
pub struct AppState {
    pub engine: Arc<gitpurge_core::Engine>,
    pub tasks: Mutex<HashMap<String, oneshot::Sender<()>>>,
}

fn main() {
    // 1. Load config
    let config = gitpurge_core::Config::load(None).unwrap_or_default();

    // 2. Initialize Engine
    let engine =
        Arc::new(gitpurge_core::Engine::open(config).expect("Failed to initialize Engine"));

    // 3. Build state
    let state = AppState {
        engine,
        tasks: Mutex::new(HashMap::new()),
    };

    // 4. Build and run Tauri app
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::repo_list,
            commands::repo_add,
            commands::repo_remove,
            commands::repo_show,
            commands::scan,
            commands::plan,
            commands::backup_create,
            commands::backup_list,
            commands::backup_show,
            commands::backup_verify,
            commands::backup_prune,
            commands::delete_branches,
            commands::archive_branches,
            commands::restore,
            commands::diff,
            commands::show_tree,
            commands::report_generate,
            commands::history_get,
            commands::history_runs_get,
            commands::auth_add,
            commands::auth_list,
            commands::auth_remove,
            commands::auth_test,
            commands::settings_get,
            commands::settings_save,
            commands::settings_export,
            commands::settings_import,
            commands::install_cli,
            commands::cancel,
            commands::save_file,
            commands::open_url,
        ])
        .setup(|_app| {
            // On Linux, the Ubuntu/GNOME taskbar shows the GTK program name.
            // Without this, it defaults to the binary name ("gitpurge-desktop").
            #[cfg(target_os = "linux")]
            gtk::glib::set_application_name("Git Purge");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
