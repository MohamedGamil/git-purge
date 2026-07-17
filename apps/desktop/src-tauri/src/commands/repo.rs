use std::path::PathBuf;
use tauri::State;

use gitpurge_core::model::{RepoId, Repository};

use super::{
    map_error, map_repo_detail, map_repo_summary, RepoDetail, RepoSummary, SerializableError,
};
use crate::AppState;

#[tauri::command]
pub async fn repo_list(state: State<'_, AppState>) -> Result<Vec<RepoSummary>, SerializableError> {
    let engine = &state.engine;
    let repos = engine.list_repos().map_err(map_error)?;
    let summaries = repos
        .into_iter()
        .map(|r| map_repo_summary(engine, &r))
        .collect();
    Ok(summaries)
}

#[tauri::command]
pub async fn repo_add(
    state: State<'_, AppState>,
    path: Option<String>,
    url: Option<String>,
    name: Option<String>,
) -> Result<RepoSummary, SerializableError> {
    let engine = &state.engine;
    let repo = if let Some(p) = path {
        Repository::new_local(PathBuf::from(p)).map_err(map_error)?
    } else if let Some(u) = url {
        let git_url = gitpurge_core::model::GitUrl::parse(&u).map_err(map_error)?;
        Repository::new_remote(git_url).map_err(map_error)?
    } else {
        return Err(SerializableError {
            code: "CONFIG".to_string(),
            message: "Either path or remote url must be supplied to add repository.".to_string(),
            hint: None,
        });
    };

    let mut repo = repo;
    if let Some(n) = name {
        repo.display_name = n;
    }

    engine.add_repo(repo.clone()).map_err(map_error)?;
    let _ = engine.save_config(None);
    Ok(map_repo_summary(engine, &repo))
}

#[tauri::command]
pub async fn repo_remove(
    state: State<'_, AppState>,
    repo_id: String,
    drop_backups: Option<bool>,
) -> Result<(), SerializableError> {
    let engine = &state.engine;
    let id = RepoId(repo_id);
    if drop_backups.unwrap_or(false) {
        let _ = engine.purge_repo_backups(&id);
    }
    engine.remove_repo(&id).map_err(map_error)?;
    let _ = engine.save_config(None);
    Ok(())
}

#[tauri::command]
pub async fn repo_show(
    state: State<'_, AppState>,
    repo_id: String,
) -> Result<RepoDetail, SerializableError> {
    let engine = &state.engine;
    let id = RepoId(repo_id);
    let repo = engine
        .get_repo(&id)
        .map_err(map_error)?
        .ok_or_else(|| SerializableError {
            code: "NOT_FOUND".to_string(),
            message: format!("Repository not found: {}", id.0),
            hint: None,
        })?;
    Ok(map_repo_detail(engine, &repo))
}
