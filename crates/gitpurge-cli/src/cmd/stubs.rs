//! Stub command handlers for commands implemented in later phases (Phase P5/P6).

use gitpurge_core::{Engine, Result, GitPurgeError};

pub fn handle_report(
    _engine: &Engine,
    _repo: &gitpurge_core::model::RepoId,
    _json: bool,
) -> Result<()> {
    Err(GitPurgeError::Other(
        "The 'report' command is not yet implemented (Phase P5).".to_string(),
    ))
}

pub fn handle_history(
    _engine: &Engine,
    _repo: &gitpurge_core::model::RepoId,
    _json: bool,
) -> Result<()> {
    Err(GitPurgeError::Other(
        "The 'history' command is not yet implemented (Phase P5).".to_string(),
    ))
}

pub fn handle_auth(
    _engine: &Engine,
    _action: &crate::cli::AuthAction,
    _json: bool,
) -> Result<()> {
    Err(GitPurgeError::Other(
        "The 'auth' command is not yet implemented (Phase P6).".to_string(),
    ))
}

pub fn handle_ui() -> Result<()> {
    Err(GitPurgeError::Other(
        "The 'ui' command is not yet implemented (Phase P4).".to_string(),
    ))
}

pub fn handle_completions(_shell: &str) -> Result<()> {
    Err(GitPurgeError::Other(
        "Shell completions generation is not yet implemented.".to_string(),
    ))
}

pub fn handle_install_cli() -> Result<()> {
    Err(GitPurgeError::Other(
        "Installing CLI tool on PATH is not yet implemented.".to_string(),
    ))
}
