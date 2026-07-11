//! Subcommand handler for `repo` command group (CLI Spec §8.1).

use std::path::Path;
use comfy_table::Table;
use gitpurge_core::{Engine, GitPurgeError, Result, model::{RepoId, Repository, GitUrl, Branch, BranchName, BranchScope}};
use serde_json::json;

pub fn handle(
    engine: &Engine,
    config_path: Option<&Path>,
    json_output: bool,
    execute: bool,
    action: &crate::cli::RepoAction,
) -> Result<()> {
    match action {
        crate::cli::RepoAction::Add {
            path_or_url,
            id,
            name,
            default_branch,
            remote: _,
        } => {
            let path = Path::new(path_or_url);
            let mut repo = if path.exists() {
                Repository::new_local(path.to_path_buf())?
            } else {
                let git_url = GitUrl::parse(path_or_url)?;
                Repository::new_remote(git_url)?
            };

            // Apply overrides
            if let Some(id_val) = id {
                repo.id = RepoId(id_val.clone());
            }
            if let Some(name_val) = name {
                repo.display_name = name_val.clone();
            }
            if let Some(ref db) = default_branch {
                let placeholder_commit = gitpurge_core::model::Commit {
                    oid: gitpurge_core::model::Oid("0000000000000000000000000000000000000000".to_string()),
                    short: "0000000".to_string(),
                    author: gitpurge_core::model::Signature {
                        name: "System".to_string(),
                        email: "system@gitpurge".to_string(),
                        when: time::OffsetDateTime::now_utc(),
                    },
                    committer: gitpurge_core::model::Signature {
                        name: "System".to_string(),
                        email: "system@gitpurge".to_string(),
                        when: time::OffsetDateTime::now_utc(),
                    },
                    author_date: time::OffsetDateTime::now_utc(),
                    commit_date: time::OffsetDateTime::now_utc(),
                    subject: "Placeholder".to_string(),
                    parents: Vec::new(),
                };
                repo.default_branch = Some(Branch {
                    name: BranchName(db.clone()),
                    scope: BranchScope::Local,
                    remote: None,
                    full_ref: format!("refs/heads/{}", db),
                    tip: placeholder_commit,
                    upstream: None,
                    is_head: false,
                });
            }

            engine.add_repo(repo.clone())?;
            engine.save_config(config_path)?;

            if json_output {
                println!(
                    "{}",
                    json!({
                        "schema_version": "1",
                        "command": "repo add",
                        "ok": true,
                        "dry_run": false,
                        "repo": repo.id.0,
                        "data": repo,
                        "warnings": [],
                        "error": null
                    })
                );
            } else {
                let loc = repo
                    .local_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| repo.remote_url.as_ref().map(|u| u.raw.clone()).unwrap_or_default());
                println!(
                    "Added repo '{}' → {} (default branch: {})",
                    repo.id.0,
                    loc,
                    repo.default_branch
                        .as_ref()
                        .map(|b| b.name.0.as_str())
                        .unwrap_or("auto-detect")
                );
            }
        }
        crate::cli::RepoAction::List => {
            let repos = engine.list_repos()?;

            if json_output {
                println!(
                    "{}",
                    json!({
                        "schema_version": "1",
                        "command": "repo list",
                        "ok": true,
                        "dry_run": false,
                        "repo": "",
                        "data": repos,
                        "warnings": [],
                        "error": null
                    })
                );
            } else {
                if repos.is_empty() {
                    println!("No repositories tracked yet. Run `git-purge repo add <path-or-url>` to track one.");
                    return Ok(());
                }
                let mut table = Table::new();
                table.set_header(vec!["ID", "NAME", "LOCATION", "DEFAULT", "BRANCHES"]);

                for repo in repos {
                    let loc = repo
                        .local_path
                        .as_ref()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|| repo.remote_url.as_ref().map(|u| u.raw.clone()).unwrap_or_default());
                    let def_branch = repo
                        .default_branch
                        .as_ref()
                        .map(|b| b.name.0.as_str())
                        .unwrap_or("auto-detect");

                    // Get branch count if scanned, otherwise "-"
                    let branches_str = repo.last_scanned_at.map(|_| "scanned").unwrap_or("-");

                    table.add_row(vec![
                        repo.id.0.as_str(),
                        repo.display_name.as_str(),
                        loc.as_str(),
                        def_branch,
                        branches_str,
                    ]);
                }
                println!("{}", table);
            }
        }
        crate::cli::RepoAction::Show { id } => {
            let repo_id = RepoId(id.clone());
            let repo = engine
                .get_repo(&repo_id)?
                .ok_or_else(|| GitPurgeError::RepoNotFound(format!("Repository not found: {}", id)))?;

            if json_output {
                println!(
                    "{}",
                    json!({
                        "schema_version": "1",
                        "command": "repo show",
                        "ok": true,
                        "dry_run": false,
                        "repo": repo.id.0,
                        "data": repo,
                        "warnings": [],
                        "error": null
                    })
                );
            } else {
                println!("Repository ID:   {}", repo.id.0);
                println!("Display Name:    {}", repo.display_name);
                if let Some(ref lp) = repo.local_path {
                    println!("Local Path:      {}", lp.to_string_lossy());
                }
                if let Some(ref ru) = repo.remote_url {
                    println!("Remote URL:      {}", ru.raw);
                }
                println!(
                    "Default Branch:  {}",
                    repo.default_branch
                        .as_ref()
                        .map(|b| b.name.0.as_str())
                        .unwrap_or("auto-detect")
                );
                println!("Provider Hint:   {:?}", repo.provider);
                println!("Added At:        {}", repo.added_at);
                if let Some(ls) = repo.last_scanned_at {
                    println!("Last Scanned At: {}", ls);
                } else {
                    println!("Last Scanned At: never");
                }
            }
        }
        crate::cli::RepoAction::Remove { id, purge_backups } => {
            let repo_id = RepoId(id.clone());
            let _repo = engine
                .get_repo(&repo_id)?
                .ok_or_else(|| GitPurgeError::RepoNotFound(format!("Repository not found: {}", id)))?;

            if !execute {
                println!("[DRY-RUN] Would remove repo '{}' from tracked list.", id);
                if *purge_backups {
                    println!("[DRY-RUN] Would purge backups bare mirror for repo '{}'.", id);
                }
                println!("Run with --execute to apply changes.");
            } else {
                // Remove from engine and config
                engine.remove_repo(&repo_id)?;

                if *purge_backups {
                    engine.purge_repo_backups(&repo_id)?;
                }

                engine.save_config(config_path)?;

                if json_output {
                    println!(
                        "{}",
                        json!({
                            "schema_version": "1",
                            "command": "repo remove",
                            "ok": true,
                            "dry_run": false,
                            "repo": id,
                            "data": { "removed": true, "backups_purged": *purge_backups },
                            "warnings": [],
                            "error": null
                        })
                    );
                } else {
                    println!("Removed repo '{}' from tracked list.", id);
                    if *purge_backups {
                        println!("Purged backups bare mirror for repo '{}'.", id);
                    }
                }
            }
        }
        crate::cli::RepoAction::SetDefault { id } => {
            let repo_id = RepoId(id.clone());
            let _ = engine
                .get_repo(&repo_id)?
                .ok_or_else(|| GitPurgeError::RepoNotFound(format!("Repository not found: {}", id)))?;

            engine.set_default_repo(repo_id)?;
            engine.save_config(config_path)?;

            if json_output {
                println!(
                    "{}",
                    json!({
                        "schema_version": "1",
                        "command": "repo set-default",
                        "ok": true,
                        "dry_run": false,
                        "repo": id,
                        "data": { "default_repo": id },
                        "warnings": [],
                        "error": null
                    })
                );
            } else {
                println!("Set default repo to '{}'.", id);
            }
        }
    }
    Ok(())
}
