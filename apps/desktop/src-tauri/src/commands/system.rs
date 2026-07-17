use tauri::State;

use super::SerializableError;
use crate::AppState;

#[tauri::command]
pub async fn install_cli(
    state: State<'_, AppState>,
    scope: String,
) -> Result<serde_json::Value, SerializableError> {
    let _ = (state, scope);
    Err(SerializableError {
        code: "UNSUPPORTED".to_string(),
        message: "Install CLI not yet implemented".to_string(),
        hint: None,
    })
}

#[tauri::command]
pub fn save_file(path: String, content: String) -> Result<(), SerializableError> {
    std::fs::write(&path, content).map_err(|e| SerializableError {
        code: "WRITE_ERROR".to_string(),
        message: format!("Failed to write file: {}", e),
        hint: Some("Verify folder permissions and space".to_string()),
    })
}

#[tauri::command]
pub fn open_url(url: String) -> Result<(), SerializableError> {
    #[cfg(target_os = "windows")]
    let res = std::process::Command::new("cmd")
        .args(["/C", "start", &url])
        .spawn();

    #[cfg(target_os = "macos")]
    let res = std::process::Command::new("open").arg(&url).spawn();

    #[cfg(target_os = "linux")]
    let res = std::process::Command::new("xdg-open").arg(&url).spawn();

    res.map(|_| ()).map_err(|e| SerializableError {
        code: "OPEN_ERROR".to_string(),
        message: format!("Failed to open URL: {}", e),
        hint: Some(e.to_string()),
    })
}
