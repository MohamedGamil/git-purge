//! Auth command handler (CLI Spec §8.12, §4.2).

use gitpurge_core::{auth::CredentialKind, model::RepoId, Engine, GitPurgeError, Result};
use serde_json::json;

/// Handle the `auth` command.
pub fn handle_auth(
    engine: &Engine,
    action: &crate::cli::AuthAction,
    json_output: bool,
) -> Result<()> {
    match action {
        crate::cli::AuthAction::Add {
            host,
            method,
            username,
            key,
            token_stdin,
        } => {
            let host_str = host.as_deref().unwrap_or("github.com");
            let method_arg = method.ok_or_else(|| {
                GitPurgeError::Config("Missing --method argument for auth add".to_string())
            })?;

            let kind = match method_arg {
                crate::cli::AuthMethodArg::Ssh => CredentialKind::SshKey,
                crate::cli::AuthMethodArg::Https => CredentialKind::HttpsBasic,
                crate::cli::AuthMethodArg::Token => CredentialKind::HttpsToken,
            };

            let secret_bytes = if *token_stdin {
                use std::io::Read;
                let mut buffer = Vec::new();
                std::io::stdin().read_to_end(&mut buffer).map_err(|e| {
                    GitPurgeError::Other(format!("Failed to read token from stdin: {}", e))
                })?;
                let trimmed = String::from_utf8_lossy(&buffer).trim().as_bytes().to_vec();
                trimmed
            } else {
                match kind {
                    CredentialKind::HttpsBasic | CredentialKind::HttpsToken => {
                        let prompt_msg = if kind == CredentialKind::HttpsToken {
                            "Enter personal access token"
                        } else {
                            "Enter password"
                        };
                        let password = dialoguer::Password::new()
                            .with_prompt(prompt_msg)
                            .interact()
                            .map_err(|e| {
                                GitPurgeError::Other(format!(
                                    "Failed to read password/token: {}",
                                    e
                                ))
                            })?;
                        password.into_bytes()
                    }
                    CredentialKind::SshKey => {
                        if dialoguer::Confirm::new()
                            .with_prompt("Does this SSH key have a passphrase?")
                            .default(false)
                            .interact()
                            .unwrap_or(false)
                        {
                            let password = dialoguer::Password::new()
                                .with_prompt("Enter SSH key passphrase")
                                .interact()
                                .map_err(|e| {
                                    GitPurgeError::Other(format!(
                                        "Failed to read passphrase: {}",
                                        e
                                    ))
                                })?;
                            password.into_bytes()
                        } else {
                            Vec::new()
                        }
                    }
                    _ => Vec::new(),
                }
            };

            let dummy_repo = RepoId(host_str.to_string());
            engine.auth_store(&dummy_repo, "origin", kind, &secret_bytes)?;

            if json_output {
                println!(
                    "{}",
                    json!({
                        "schema_version": "1",
                        "command": "auth add",
                        "ok": true,
                        "dry_run": false,
                        "repo": host_str,
                        "data": {
                            "host": host_str,
                            "method": format!("{:?}", method_arg).to_lowercase(),
                            "username": username.as_deref().unwrap_or(""),
                            "key_path": key.as_deref().unwrap_or(""),
                        },
                        "warnings": [],
                        "error": null
                    })
                );
            } else {
                println!("Stored credential for {} in OS keychain.", host_str);
            }
        }
        crate::cli::AuthAction::List => {
            let list = engine.auth_list()?;

            if json_output {
                println!(
                    "{}",
                    json!({
                        "schema_version": "1",
                        "command": "auth list",
                        "ok": true,
                        "dry_run": false,
                        "data": list,
                        "warnings": [],
                        "error": null
                    })
                );
            } else {
                if list.is_empty() {
                    println!("No stored credentials found.");
                    return Ok(());
                }

                let mut table = comfy_table::Table::new();
                table.set_header(vec!["METHOD", "LABEL"]);
                for cred in list {
                    let method_str = format!("{:?}", cred.kind);
                    table.add_row(vec![method_str.as_str(), cred.label.as_str()]);
                }
                println!("{}", table);
            }
        }
        crate::cli::AuthAction::Remove { id } => {
            let dummy_repo = RepoId(id.clone());
            engine.auth_remove(&dummy_repo, "origin")?;

            if json_output {
                println!(
                    "{}",
                    json!({
                        "schema_version": "1",
                        "command": "auth remove",
                        "ok": true,
                        "dry_run": false,
                        "data": { "id": id },
                        "warnings": [],
                        "error": null
                    })
                );
            } else {
                println!("Removed credential for {}.", id);
            }
        }
        crate::cli::AuthAction::Test { host } => {
            let host_str = host.as_deref().unwrap_or("github.com");
            let dummy_repo = RepoId(host_str.to_string());
            let success = engine.auth_test(&dummy_repo, "origin")?;

            if json_output {
                println!(
                    "{}",
                    json!({
                        "schema_version": "1",
                        "command": "auth test",
                        "ok": true,
                        "dry_run": false,
                        "data": {
                            "host": host_str,
                            "success": success,
                        },
                        "warnings": [],
                        "error": null
                    })
                );
            } else {
                if success {
                    println!("✓ {}: authenticated successfully (mocked).", host_str);
                } else {
                    println!("✗ {}: authentication failed (mocked).", host_str);
                }
            }
        }
    }

    Ok(())
}
