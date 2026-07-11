//! `git-purge` CLI — thin adapter over `gitpurge-core` (CONVENTIONS §2).
//!
//! This binary translates CLI arguments into `gitpurge-core::Engine` calls and renders
//! the results. **No git/DB/keychain logic lives here** — only argument parsing,
//! output formatting, and exit-code mapping.

use clap::{Parser, Subcommand};

/// Git Purge — safely purge stale branches with a net under every operation.
#[derive(Debug, Parser)]
#[command(
    name = "git-purge",
    version,
    about = "Safely purge stale git branches — with a net under every operation.",
    long_about = "Git Purge is a CLI-first utility for safely cleaning up old and stale \
                  branches from git repositories. Every destructive operation is dry-run by \
                  default, backed up before execution, and easily restorable.",
    after_help = "Use `git-purge <command> --help` for detailed help on each command."
)]
struct Cli {
    /// Subcommand to run.
    #[command(subcommand)]
    command: Option<Commands>,

    /// Output as JSON instead of human-readable tables.
    #[arg(long, global = true)]
    json: bool,

    /// Suppress colored output.
    #[arg(long, global = true)]
    no_color: bool,

    /// Path to config file (default: auto-resolved via XDG/KnownFolders).
    #[arg(long, global = true)]
    config: Option<String>,

    /// Target repository (id or path). Defaults to the current directory.
    #[arg(long, global = true)]
    repo: Option<String>,

    /// Increase verbosity (-v, -vv).
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Suppress all output except errors.
    #[arg(short, long, global = true)]
    quiet: bool,
}

/// Top-level subcommands (CONVENTIONS §9).
#[derive(Debug, Subcommand)]
enum Commands {
    /// Manage tracked repositories.
    Repo {
        /// Repo subcommand.
        #[command(subcommand)]
        action: RepoAction,
    },
    /// Classify branches (read-only scan).
    Scan,
    /// Show what delete/archive would do (dry-run).
    Plan,
    /// Manage backup snapshots.
    Backup {
        /// Backup subcommand.
        #[command(subcommand)]
        action: BackupAction,
    },
    /// Delete stale/merged branches (dry-run default).
    Delete,
    /// Archive unmerged branches into a legacy branch.
    Archive,
    /// Restore a branch or tag from a snapshot.
    Restore,
    /// Compare two branches.
    Diff,
    /// View repo/file content at a ref/commit.
    Show,
    /// Generate audit and trend reports.
    Report,
    /// View trend history.
    History,
    /// Manage authentication credentials.
    Auth {
        /// Auth subcommand.
        #[command(subcommand)]
        action: AuthAction,
    },
    /// Launch the desktop UI (if installed).
    Ui,
    /// Install git-purge on PATH.
    InstallCli,
    /// Generate shell completions.
    Completions {
        /// Shell to generate completions for.
        shell: String,
    },
}

/// Repo subcommands.
#[derive(Debug, Subcommand)]
enum RepoAction {
    /// Add a repository to track.
    Add,
    /// List tracked repositories.
    List,
    /// Remove a tracked repository.
    Remove,
    /// Show details of a tracked repository.
    Show,
}

/// Backup subcommands.
#[derive(Debug, Subcommand)]
enum BackupAction {
    /// Create a new backup snapshot.
    Create,
    /// List existing snapshots.
    List,
    /// Show snapshot details.
    Show,
    /// Verify snapshot integrity.
    Verify,
    /// Prune old snapshots.
    Prune,
}

/// Auth subcommands.
#[derive(Debug, Subcommand)]
enum AuthAction {
    /// Add a credential.
    Add,
    /// List stored credentials.
    List,
    /// Remove a credential.
    Remove,
    /// Test a credential.
    Test,
}

fn main() {
    let cli = Cli::parse();

    // TODO(P3): initialize tracing, load config, open Engine, dispatch subcommands.
    match &cli.command {
        Some(_cmd) => {
            eprintln!("git-purge: command not yet implemented (phase P3)");
            std::process::exit(2);
        }
        None => {
            // No subcommand — show help.
            use clap::CommandFactory;
            Cli::command().print_help().ok();
            println!();
        }
    }
}
