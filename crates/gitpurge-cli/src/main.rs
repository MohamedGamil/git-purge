//! `git-purge` CLI — thin adapter over `gitpurge-core` (CONVENTIONS §2).

use clap::Parser;
use gitpurge_core::{
    model::{GitUrl, RepoId, Repository},
    Engine, GitPurgeError, Result,
};
use std::path::Path;

mod cli;
mod cmd;
mod confirm;
mod exit;

fn run() -> Result<()> {
    let args = cli::Cli::parse();

    // 1. Resolve config path and load configuration
    let config_path = args.config.as_deref().map(Path::new);
    let config = gitpurge_core::config::Config::load(config_path)?;

    // 2. Open the engine
    let engine = Engine::open(config)?;

    // 4. Dispatch subcommands
    match &args.command {
        Some(cli::Commands::Repo { action }) => {
            cmd::repo::handle(&engine, config_path, args.json, args.execute, action)?;
        }
        Some(cli::Commands::Scan {
            no_refresh,
            filters,
        }) => {
            let repo_id = resolve_repo(&engine, config_path, args.repo.as_deref())?;
            cmd::scan::handle_scan(&engine, &repo_id, *no_refresh, filters, args.json)?;
        }
        Some(cli::Commands::Plan { action, filters }) => {
            let repo_id = resolve_repo(&engine, config_path, args.repo.as_deref())?;
            cmd::scan::handle_plan(&engine, &repo_id, action, filters, args.json)?;
        }
        Some(cli::Commands::Backup { action }) => {
            let repo_id = resolve_repo(&engine, config_path, args.repo.as_deref())?;
            cmd::backup::handle_backup(&engine, &repo_id, action, args.execute, args.json)?;
        }
        Some(cli::Commands::Delete {
            include_unmerged,
            no_backup,
            force_unmerged,
            continue_on_error,
            filters,
        }) => {
            let repo_id = resolve_repo(&engine, config_path, args.repo.as_deref())?;
            cmd::delete::handle_delete(
                &engine,
                &repo_id,
                args.execute,
                *no_backup,
                args.yes,
                *force_unmerged,
                *include_unmerged,
                filters.unmerged,
                *continue_on_error,
                filters,
                args.json,
            )?;
        }
        Some(cli::Commands::Archive {
            target,
            strategy,
            push,
            filters,
        }) => {
            let repo_id = resolve_repo(&engine, config_path, args.repo.as_deref())?;
            cmd::delete::handle_archive(
                &engine,
                &repo_id,
                args.execute,
                target,
                strategy,
                *push,
                args.yes,
                filters,
                args.json,
            )?;
        }
        Some(cli::Commands::Restore {
            snapshot_id,
            ref_or_glob,
            as_tag,
            as_name,
            target: _,
            force,
        }) => {
            let repo_id = resolve_repo(&engine, config_path, args.repo.as_deref())?;
            cmd::backup::handle_restore(
                &engine,
                &repo_id,
                snapshot_id,
                ref_or_glob,
                *as_tag,
                as_name,
                *force,
                args.execute,
                args.json,
            )?;
        }
        Some(cli::Commands::Diff {
            ref_a,
            ref_b,
            stat,
            name_only,
            patch,
        }) => {
            let repo_id = resolve_repo(&engine, config_path, args.repo.as_deref())?;
            cmd::diff::handle_diff(
                &engine, &repo_id, ref_a, ref_b, *stat, *name_only, *patch, args.json,
            )?;
        }
        Some(cli::Commands::Show { ref_spec, path }) => {
            let repo_id = resolve_repo(&engine, config_path, args.repo.as_deref())?;
            cmd::diff::handle_show(&engine, &repo_id, ref_spec, path, args.json)?;
        }
        Some(cli::Commands::Report {
            r#type,
            format,
            out,
            baseline,
            filters: _,
        }) => {
            let repo_id = resolve_repo(&engine, config_path, args.repo.as_deref())?;
            cmd::reporting::handle_report(
                &engine,
                &repo_id,
                *r#type,
                *format,
                out.clone(),
                baseline.clone(),
                args.json,
            )?;
        }
        Some(cli::Commands::History {
            action,
            limit,
            metric,
            since,
        }) => {
            if let Some(cli::HistoryAction::Import { path, map }) = action {
                cmd::reporting::handle_history_import(&engine, path, map, args.execute, args.json)?;
            } else {
                let repo_id = resolve_repo(&engine, config_path, args.repo.as_deref())?;
                cmd::reporting::handle_history(
                    &engine,
                    &repo_id,
                    *limit,
                    metric.clone(),
                    since.clone(),
                    args.json,
                )?;
            }
        }
        Some(cli::Commands::Auth { action }) => {
            cmd::auth::handle_auth(&engine, config_path, action, args.json)?;
        }
        Some(cli::Commands::Ui) => {
            let repo_id = resolve_repo(&engine, config_path, args.repo.as_deref()).ok();
            cmd::ui::handle_ui(repo_id.as_ref())?;
        }
        Some(cli::Commands::InstallCli {
            user,
            system,
            dir,
            force,
        }) => {
            cmd::install_cli::handle_install_cli(
                *user,
                *system,
                dir.clone(),
                *force,
                args.execute,
            )?;
        }
        Some(cli::Commands::Completions { shell }) => {
            cmd::completions::handle_completions(*shell)?;
        }
        None => {
            use clap::CommandFactory;
            cli::Cli::command().print_help().ok();
            println!();
        }
    }

    Ok(())
}

fn resolve_repo(
    engine: &Engine,
    config_path: Option<&Path>,
    repo_arg: Option<&str>,
) -> Result<RepoId> {
    if let Some(arg) = repo_arg {
        let repo_id = RepoId(arg.to_string());
        if engine.get_repo(&repo_id)?.is_some() {
            return Ok(repo_id);
        }

        let repos = engine.list_repos()?;
        for r in &repos {
            if r.display_name == arg {
                return Ok(r.id.clone());
            }
            if let Some(ref lp) = r.local_path {
                if lp.to_string_lossy() == arg {
                    return Ok(r.id.clone());
                }
            }
        }

        let path = Path::new(arg);
        let repo = if path.exists() {
            Repository::new_local(path.to_path_buf())?
        } else if arg.contains("://") || arg.contains('@') {
            let git_url = GitUrl::parse(arg)?;
            Repository::new_remote(git_url)?
        } else {
            return Err(GitPurgeError::RepoNotFound(format!(
                "Repository '{}' not found and is not a valid path or Git URL.",
                arg
            )));
        };

        let registered_id = repo.id.clone();
        engine.add_repo(repo)?;
        engine.save_config(config_path)?;
        return Ok(registered_id);
    }

    if let Some(id) = engine.default_repo_id() {
        return Ok(id);
    }

    let cwd = std::env::current_dir().map_err(|e| {
        GitPurgeError::Config(format!(
            "Failed to resolve current working directory: {}",
            e
        ))
    })?;

    let mut git_dir = cwd.clone();
    let is_git = loop {
        if git_dir.join(".git").exists() {
            break true;
        }
        if let Some(parent) = git_dir.parent() {
            git_dir = parent.to_path_buf();
        } else {
            break false;
        }
    };

    if !is_git {
        return Err(GitPurgeError::RepoNotFound(
            "No repository specified, no default repository configured, and current directory is not inside a Git repository.".to_string()
        ));
    }

    let repo = Repository::new_local(git_dir)?;
    let registered_id = repo.id.clone();
    if engine.get_repo(&registered_id)?.is_none() {
        engine.add_repo(repo)?;
        engine.save_config(config_path)?;
    }
    Ok(registered_id)
}

fn main() {
    if let Err(err) = run() {
        exit::handle_error(err);
    }
}
