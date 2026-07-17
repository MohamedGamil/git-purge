use std::path::PathBuf;
use tauri::State;

use gitpurge_core::model::GlobPattern;

use super::{map_error, map_settings, SerializableError, Settings};
use crate::AppState;

#[tauri::command]
pub async fn settings_get(state: State<'_, AppState>) -> Result<Settings, SerializableError> {
    let config = state.engine.config();
    Ok(map_settings(&config))
}

#[tauri::command]
pub async fn settings_save(
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<Settings, SerializableError> {
    let engine = &state.engine;
    let mut config = engine.config();

    // Map fields back
    config.default_policy.age = settings.policy.age;
    if !settings.policy.naming_regex.is_empty() {
        config.default_policy.naming.allowed = vec![gitpurge_core::model::RegexSource(
            settings.policy.naming_regex.clone(),
        )];
    } else {
        config.default_policy.naming.allowed = Vec::new();
    }

    config.default_policy.protection.protected_globs = settings
        .policy
        .protected_refs
        .iter()
        .map(|g| GlobPattern(g.clone()))
        .collect();

    config.default_policy.excludes = settings
        .policy
        .exclude_globs
        .iter()
        .map(|g| GlobPattern(g.clone()))
        .collect();

    if !settings.backups_root.is_empty() {
        config.backups_root = Some(PathBuf::from(settings.backups_root));
    } else {
        config.backups_root = None;
    }

    config.date_format = if settings.date_format.trim().is_empty() {
        "YYYY-MM-DD h:m a".to_string()
    } else {
        settings.date_format.clone()
    };

    // Save to engine and disk
    engine.update_config(config.clone());
    engine.save_config(None).map_err(map_error)?;

    Ok(map_settings(&config))
}

#[tauri::command]
pub async fn settings_export(
    state: State<'_, AppState>,
    path: String,
) -> Result<(), SerializableError> {
    let engine = &state.engine;
    let export_path = std::path::Path::new(&path);
    engine.save_config(Some(export_path)).map_err(map_error)?;
    Ok(())
}

#[tauri::command]
pub async fn settings_import(
    state: State<'_, AppState>,
    path: String,
) -> Result<Settings, SerializableError> {
    let engine = &state.engine;
    let import_path = std::path::Path::new(&path);
    let new_config = gitpurge_core::Config::load(Some(import_path)).map_err(map_error)?;
    engine.update_config(new_config.clone());
    engine.save_config(None).map_err(map_error)?;
    Ok(map_settings(&new_config))
}
