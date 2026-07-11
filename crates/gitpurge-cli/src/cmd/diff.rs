//! Subcommand handlers for `diff` and `show` (CLI Spec §8.8, §8.9).

use std::path::Path;
use gitpurge_core::{
    Engine, Result, GitPurgeError,
    model::{RepoId, RefSpec, BranchName},
};
use serde_json::json;

pub fn parse_ref_spec(s: &str) -> RefSpec {
    if s.starts_with("refs/heads/") {
        RefSpec::Branch(BranchName(s["refs/heads/".len()..].to_string()))
    } else if s.starts_with("refs/tags/") {
        RefSpec::Tag(s["refs/tags/".len()..].to_string())
    } else if s.starts_with("tags/") {
        RefSpec::Tag(s["tags/".len()..].to_string())
    } else if s.starts_with("heads/") {
        RefSpec::Branch(BranchName(s["heads/".len()..].to_string()))
    } else if s == "HEAD" {
        RefSpec::Symbolic(s.to_string())
    } else if s.len() == 40 && s.chars().all(|c| c.is_ascii_hexdigit()) {
        RefSpec::Oid(gitpurge_core::model::Oid(s.to_string()))
    } else {
        RefSpec::Branch(BranchName(s.to_string()))
    }
}

pub fn handle_diff(
    engine: &Engine,
    repo_id: &RepoId,
    ref_a: &str,
    ref_b: &str,
    stat: bool,
    name_only: bool,
    patch: bool,
    json_output: bool,
) -> Result<()> {
    let spec_a = parse_ref_spec(ref_a);
    let spec_b = parse_ref_spec(ref_b);

    let diff_result = engine.diff(repo_id, &spec_a, &spec_b)?;

    if json_output {
        println!(
            "{}",
            json!({
                "schema_version": "1",
                "command": "diff",
                "ok": true,
                "dry_run": false,
                "repo": repo_id.0,
                "data": diff_result,
                "warnings": [],
                "error": null
            })
        );
        return Ok(());
    }

    if name_only {
        for entry in &diff_result.entries {
            println!("{}", entry.path);
        }
    } else if patch {
        println!("Patch format (unified diff) is only available via git directly. Showing diff stat:");
        print_diff_stat(&diff_result);
    } else if stat {
        print_diff_stat(&diff_result);
    }

    Ok(())
}

fn print_diff_stat(diff: &gitpurge_core::diff::DiffResult) {
    for entry in &diff.entries {
        let kind_char = match entry.kind {
            gitpurge_core::diff::DiffKind::Added => "A",
            gitpurge_core::diff::DiffKind::Deleted => "D",
            gitpurge_core::diff::DiffKind::Modified => "M",
            gitpurge_core::diff::DiffKind::Renamed => "R",
            gitpurge_core::diff::DiffKind::Copied => "C",
        };
        let plus_minus = format!("+{}/-{}", entry.additions, entry.deletions);
        println!("{:5} {:40} | {}", kind_char, entry.path, plus_minus);
    }
    println!(
        "\n{} files changed, {} insertions(+), {} deletions(-)",
        diff.files_changed, diff.insertions, diff.deletions
    );
}

pub fn handle_show(
    engine: &Engine,
    repo_id: &RepoId,
    ref_spec: &str,
    path: &Option<String>,
    json_output: bool,
) -> Result<()> {
    let spec = parse_ref_spec(ref_spec);

    if let Some(ref file_path) = path {
        let content = engine.show_file(repo_id, &spec, Path::new(file_path))?;
        if json_output {
            println!(
                "{}",
                json!({
                    "schema_version": "1",
                    "command": "show file",
                    "ok": true,
                    "dry_run": false,
                    "repo": repo_id.0,
                    "data": {
                        "path": file_path,
                        "content_len": content.len(),
                        "content": String::from_utf8_lossy(&content)
                    },
                    "warnings": [],
                    "error": null
                })
            );
        } else {
            std::io::Write::write_all(&mut std::io::stdout(), &content)
                .map_err(|e| GitPurgeError::Git(format!("Failed to write file content to stdout: {}", e)))?;
        }
    } else {
        let tree_view = engine.show_tree(repo_id, &spec, None)?;
        if json_output {
            println!(
                "{}",
                json!({
                    "schema_version": "1",
                    "command": "show tree",
                    "ok": true,
                    "dry_run": false,
                    "repo": repo_id.0,
                    "data": tree_view,
                    "warnings": [],
                    "error": null
                })
            );
        } else {
            println!("Tree at reference '{}':", ref_spec);
            for entry in tree_view.entries {
                println!("{}", entry.path);
            }
        }
    }

    Ok(())
}
