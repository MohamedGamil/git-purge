use tauri::{AppHandle, State};

use super::SerializableError;
use crate::AppState;

#[tauri::command]
pub async fn auth_add(
    state: State<'_, AppState>,
    credential: serde_json::Value,
) -> Result<serde_json::Value, SerializableError> {
    let _ = (state, credential);
    Err(SerializableError {
        code: "UNSUPPORTED".to_string(),
        message: "Auth command not yet implemented (Phase P6)".to_string(),
        hint: None,
    })
}

#[tauri::command]
pub async fn auth_list(state: State<'_, AppState>) -> Result<serde_json::Value, SerializableError> {
    let _ = state;
    Err(SerializableError {
        code: "UNSUPPORTED".to_string(),
        message: "Auth command not yet implemented (Phase P6)".to_string(),
        hint: None,
    })
}

#[tauri::command]
pub async fn auth_remove(state: State<'_, AppState>, id: String) -> Result<(), SerializableError> {
    let _ = (state, id);
    Err(SerializableError {
        code: "UNSUPPORTED".to_string(),
        message: "Auth command not yet implemented (Phase P6)".to_string(),
        hint: None,
    })
}

#[tauri::command]
pub async fn auth_test(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    repo_id: Option<String>,
    task_id: String,
) -> Result<serde_json::Value, SerializableError> {
    let _ = (app, state, id, repo_id, task_id);
    Err(SerializableError {
        code: "UNSUPPORTED".to_string(),
        message: "Auth command not yet implemented (Phase P6)".to_string(),
        hint: None,
    })
}
