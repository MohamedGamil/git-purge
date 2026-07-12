//! UI command handler (CLI Spec §8.13).

use gitpurge_core::{model::RepoId, GitPurgeError, Result};
use std::process::Command;

/// Handle the `ui` command.
pub fn handle_ui(repo_id: Option<&RepoId>) -> Result<()> {
    let bin_name = if cfg!(windows) {
        "gitpurge-desktop.exe"
    } else {
        "gitpurge-desktop"
    };

    // Check if the binary is on PATH
    let path_var = std::env::var_os("PATH").unwrap_or_default();
    let found = std::env::split_paths(&path_var).any(|dir| {
        let path = dir.join(bin_name);
        path.is_file()
    });

    if !found {
        return Err(GitPurgeError::RepoNotFound(
            "Git Purge Desktop application is not installed. Please install the desktop application first.".to_string()
        ));
    }

    let mut cmd = Command::new(bin_name);
    if let Some(id) = repo_id {
        cmd.arg("--repo").arg(&id.0);
    }

    println!("Launching Git Purge Desktop...");
    cmd.spawn()
        .map_err(|e| GitPurgeError::Other(format!("Failed to spawn desktop process: {}", e)))?;

    Ok(())
}
