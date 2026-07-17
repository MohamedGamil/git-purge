//! Subcommand handler for `show` (CLI Spec §8.9).

use gitpurge_core::{
    model::RepoId,
    Engine, GitPurgeError, Result,
};
use serde_json::json;
use std::path::Path;
use super::diff::parse_ref_spec;

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
            std::io::Write::write_all(&mut std::io::stdout(), &content).map_err(|e| {
                GitPurgeError::Git(format!("Failed to write file content to stdout: {}", e))
            })?;
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
