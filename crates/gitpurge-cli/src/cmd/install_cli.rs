//! Install CLI command handler (CLI Spec §8.15).

use directories::BaseDirs;
use gitpurge_core::{GitPurgeError, Result};
use std::path::PathBuf;

/// Handle the `install-cli` command.
pub fn handle_install_cli(
    _user: bool,
    system: bool,
    dir_override: Option<String>,
    force: bool,
    execute: bool,
) -> Result<()> {
    // 1. Locate current running binary
    let current_exe = std::env::current_exe()
        .map_err(|e| GitPurgeError::Other(format!("Failed to locate current executable: {}", e)))?;

    // 2. Resolve destination directory
    let dest_dir = if let Some(ref d) = dir_override {
        PathBuf::from(d)
    } else if system {
        if cfg!(windows) {
            let prog_files = std::env::var_os("ProgramFiles")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(r"C:\Program Files"));
            prog_files.join("git-purge")
        } else {
            PathBuf::from("/usr/local/bin")
        }
    } else {
        // user is the default if not system or dir_override
        if cfg!(windows) {
            let base_dirs = BaseDirs::new().ok_or_else(|| {
                GitPurgeError::Config("Failed to resolve user directories on Windows".to_string())
            })?;
            base_dirs
                .data_local_dir()
                .join("Programs")
                .join("git-purge")
        } else {
            // Linux / macOS: ~/.local/bin/
            let base_dirs = BaseDirs::new().ok_or_else(|| {
                GitPurgeError::Config("Failed to resolve user directories".to_string())
            })?;
            base_dirs.home_dir().join(".local").join("bin")
        }
    };

    let bin_name = if cfg!(windows) {
        "git-purge.exe"
    } else {
        "git-purge"
    };
    let dest_path = dest_dir.join(bin_name);

    // 3. Check for existing file
    if dest_path.exists() && !force {
        return Err(GitPurgeError::Config(format!(
            "Binary already exists at '{}'. Use --force to overwrite.",
            dest_path.display()
        )));
    }

    // 4. Verify if destination folder is on PATH
    let path_var = std::env::var_os("PATH").unwrap_or_default();
    let paths = std::env::split_paths(&path_var);

    // Canonicalize paths to compare them accurately
    let dest_dir_canonical = dest_dir.canonicalize().unwrap_or_else(|_| dest_dir.clone());
    let already_on_path = paths
        .into_iter()
        .any(|p| p.canonicalize().unwrap_or_else(|_| p.clone()) == dest_dir_canonical);

    if !execute {
        println!(
            "[DRY-RUN] would copy {} → {}",
            current_exe.display(),
            dest_path.display()
        );
        if already_on_path {
            println!(
                "[DRY-RUN] {} already on PATH — no profile edit needed",
                dest_dir.display()
            );
        } else {
            if cfg!(windows) {
                println!(
                    "[DRY-RUN] {} is not on PATH — would update PATH via registry/setx",
                    dest_dir.display()
                );
            } else {
                println!("[DRY-RUN] {} is not on PATH — would append PATH export to ~/.bashrc and ~/.zshrc", dest_dir.display());
            }
        }
        println!("Run with --execute to apply.");
        return Ok(());
    }

    // 5. Execute copy
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            GitPurgeError::Io(std::io::Error::new(
                e.kind(),
                format!(
                    "Failed to create destination directory '{}': {}",
                    parent.display(),
                    e
                ),
            ))
        })?;
    }

    std::fs::copy(&current_exe, &dest_path).map_err(|e| {
        GitPurgeError::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to copy binary to '{}': {}", dest_path.display(), e),
        ))
    })?;

    // Set executable permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&dest_path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&dest_path, perms)?;
    }

    // 6. Update PATH if not already present
    if already_on_path {
        println!(
            "Installed → {} · `git purge` alias active (PATH already includes it)",
            dest_path.display()
        );
    } else {
        if cfg!(windows) {
            let mut success = false;
            if system {
                let status = std::process::Command::new("setx")
                    .args([
                        "/M",
                        "PATH",
                        &format!("{};{}", path_var.to_string_lossy(), dest_dir.display()),
                    ])
                    .status();
                if let Ok(st) = status {
                    if st.success() {
                        success = true;
                    }
                }
            } else {
                let status = std::process::Command::new("setx")
                    .args([
                        "PATH",
                        &format!("{};{}", path_var.to_string_lossy(), dest_dir.display()),
                    ])
                    .status();
                if let Ok(st) = status {
                    if st.success() {
                        success = true;
                    }
                }
            }

            if success {
                println!("Installed → {} · `git purge` alias active (PATH updated; restart shell to apply)", dest_path.display());
            } else {
                println!(
                    "Installed → {} · `git purge` alias copied.",
                    dest_path.display()
                );
                println!("Please add '{}' to your PATH manually.", dest_dir.display());
            }
        } else {
            // Unix: append to .bashrc / .zshrc
            let base_dirs = BaseDirs::new();
            let mut profiles_updated = Vec::new();

            if let Some(ref bd) = base_dirs {
                let home = bd.home_dir();
                for profile in &[".bashrc", ".zshrc"] {
                    let profile_path = home.join(profile);
                    if profile_path.exists() {
                        if let Ok(content) = std::fs::read_to_string(&profile_path) {
                            let export_line =
                                format!("export PATH=\"$PATH:{}\"", dest_dir.display());
                            if !content.contains(&export_line) {
                                use std::fs::OpenOptions;
                                use std::io::Write;
                                if let Ok(mut file) =
                                    OpenOptions::new().append(true).open(&profile_path)
                                {
                                    if writeln!(file, "\n# Added by git-purge\n{}", export_line)
                                        .is_ok()
                                    {
                                        profiles_updated.push(profile.to_string());
                                    }
                                }
                            } else {
                                profiles_updated.push(profile.to_string());
                            }
                        }
                    }
                }
            }

            if profiles_updated.is_empty() {
                println!(
                    "Installed → {} · `git purge` alias copied.",
                    dest_path.display()
                );
                println!(
                    "Please add '{}' to your PATH manually by exporting it in your shell profile.",
                    dest_dir.display()
                );
            } else {
                let profiles_str = profiles_updated.join(" and ");
                println!(
                    "Installed → {} · `git purge` alias active (PATH appended to {})",
                    dest_path.display(),
                    profiles_str
                );
            }
        }
    }

    Ok(())
}
