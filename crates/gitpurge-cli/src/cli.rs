//! Clap CLI arguments and command definitions.

use clap::{Args, Parser, Subcommand, ValueEnum};

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
pub struct Cli {
    /// Subcommand to run.
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Output as JSON instead of human-readable tables.
    #[arg(long, global = true)]
    pub json: bool,

    /// Suppress colored output.
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Path to config file (default: auto-resolved via XDG/KnownFolders).
    #[arg(long, global = true)]
    pub config: Option<String>,

    /// Target repository (id, label, or path). Defaults to the default_repo in config or current directory.
    #[arg(long, global = true)]
    pub repo: Option<String>,

    /// Assume "yes" for standard confirmations.
    #[arg(short, long, global = true)]
    pub yes: bool,

    /// Perform mutations. Absent => dry-run.
    #[arg(short = 'e', long, global = true)]
    pub execute: bool,

    /// Increase verbosity (-v, -vv).
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Suppress all output except errors.
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Override the data root directory (history DB, backups, reports).
    #[arg(long, global = true)]
    pub data_dir: Option<String>,
}

/// Top-level subcommands (CONVENTIONS §9).
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Manage tracked repositories.
    Repo {
        /// Repo subcommand.
        #[command(subcommand)]
        action: RepoAction,
    },
    /// Classify branches (read-only scan).
    Scan {
        /// Do not fetch/prune from remote before classifying.
        #[arg(long)]
        no_refresh: bool,

        #[command(flatten)]
        filters: SelectionFlags,
    },
    /// Show what delete/archive would do (dry-run).
    Plan {
        /// Which action's plan to compute (delete or archive).
        #[arg(long, value_enum, default_value = "delete")]
        action: ActionType,

        #[command(flatten)]
        filters: SelectionFlags,
    },
    /// Manage backup snapshots.
    Backup {
        /// Backup subcommand.
        #[command(subcommand)]
        action: BackupAction,
    },
    /// Delete stale/merged branches (dry-run default).
    Delete {
        /// Delete unmerged stale branches too (requires repo ID typed confirmation).
        #[arg(long)]
        include_unmerged: bool,

        /// Delete ONLY unmerged stale branches (requires repo ID typed confirmation).
        #[arg(long)]
        unmerged: bool,

        /// Skip creating the pre-op snapshot backup.
        #[arg(long)]
        no_backup: bool,

        /// Bypass the typed repository ID token confirmation prompt for unmerged execute in non-interactive scripts.
        #[arg(long)]
        force_unmerged: bool,

        /// Do not stop on the first deletion failure; continue deleting remaining branches.
        #[arg(long)]
        continue_on_error: bool,

        #[command(flatten)]
        filters: SelectionFlags,
    },
    /// Archive unmerged branches into a legacy branch.
    Archive {
        /// Target legacy branch where unmerged commits are merged.
        #[arg(long, default_value = "main-legacy")]
        target: String,

        /// Merge strategy (ours or theirs).
        #[arg(long, value_enum, default_value = "ours")]
        strategy: MergeStrategyArg,

        /// Push the target branch to the remote origin after merging.
        #[arg(long)]
        push: bool,

        #[command(flatten)]
        filters: SelectionFlags,
    },
    /// Restore a branch or tag from a snapshot.
    Restore {
        /// Snapshot ID to restore from (or 'latest').
        snapshot_id: String,

        /// Branch name or glob pattern (e.g. 'feature/x' or 'refs/heads/*').
        ref_or_glob: String,

        /// Recreate the restored reference as a tag instead of a branch.
        #[arg(long)]
        as_tag: bool,

        /// Restore under a different branch name (only valid when restoring a single branch).
        #[arg(long)]
        as_name: Option<String>,

        /// Destination target (local or remote).
        #[arg(long, value_enum, default_value = "local")]
        target: RestoreTarget,

        /// Force overwrite of existing branches/tags if they already exist in the target.
        #[arg(long)]
        force: bool,
    },
    /// Compare two branches.
    Diff {
        /// First reference (RefSpec).
        ref_a: String,
        /// Second reference (RefSpec).
        ref_b: String,

        /// Render summary stats (files changed, insertions/deletions).
        #[arg(long, default_value = "true")]
        stat: bool,

        /// List changed paths only.
        #[arg(long)]
        name_only: bool,

        /// Show full unified diff (patch).
        #[arg(short, long)]
        patch: bool,
    },
    /// View repo/file content at a ref/commit.
    Show {
        /// RefSpec (branch, tag, SHA, or snapshot:ref).
        ref_spec: String,

        /// Optional file path to view. If omitted, lists the tree.
        path: Option<String>,
    },
    /// Generate audit and trend reports.
    Report {
        /// Report type (audit, trend, or both).
        #[arg(long, value_enum, default_value = "both")]
        r#type: ReportType,

        /// Output format (md, json, html).
        #[arg(long, value_enum, default_value = "md")]
        format: ReportFormatArg,

        /// Output path for the report (directory or file).
        #[arg(long)]
        out: Option<String>,

        /// Run ID to use as a baseline for trend report.
        #[arg(long)]
        baseline: Option<String>,

        #[command(flatten)]
        filters: SelectionFlags,
    },
    /// View trend history.
    History {
        /// Cap the number of listed runs.
        #[arg(long, default_value = "20")]
        limit: u32,

        /// Focus history on a specific metric (total, active, stale, merged, unmerged, non_standard).
        #[arg(long)]
        metric: Option<String>,

        /// Limit runs to those recorded after a date or duration (e.g. '30 days ago').
        #[arg(long)]
        since: Option<String>,
    },
    /// Manage authentication credentials.
    Auth {
        /// Auth subcommand.
        #[command(subcommand)]
        action: AuthAction,
    },
    /// Launch the desktop UI (if installed).
    Ui,
    /// Install git-purge on PATH.
    InstallCli {
        /// Install for the current user only.
        #[arg(long, default_value = "true")]
        user: bool,

        /// Install for all users (requires elevated privileges).
        #[arg(long)]
        system: bool,

        /// Override install directory.
        #[arg(long)]
        dir: Option<String>,

        /// Force overwrite of existing git-purge binary on PATH.
        #[arg(long)]
        force: bool,
    },
    /// Generate shell completions.
    Completions {
        /// Shell to generate completions for.
        #[arg(value_enum)]
        shell: ShellArg,
    },
}

/// Repo subcommands.
#[derive(Debug, Subcommand)]
pub enum RepoAction {
    /// Add a repository to track.
    Add {
        /// Local directory path or remote Git URL.
        path_or_url: String,

        /// Custom identifier (default derived from folder name).
        #[arg(long)]
        id: Option<String>,

        /// Human label for the repository.
        #[arg(long)]
        name: Option<String>,

        /// Override default branch auto-detection (e.g. 'main', 'master').
        #[arg(long)]
        default_branch: Option<String>,

        /// Register an extra remote URL.
        #[arg(long)]
        remote: Option<String>,
    },
    /// List tracked repositories.
    List,
    /// Show details of a tracked repository.
    Show {
        /// Repository ID.
        id: String,
    },
    /// Remove a tracked repository.
    Remove {
        /// Repository ID.
        id: String,

        /// Also delete the repository's bare mirror under backups/ (destructive).
        #[arg(long)]
        purge_backups: bool,
    },
    /// Set a default repository.
    SetDefault {
        /// Repository ID.
        id: String,
    },
}

/// Backup subcommands.
#[derive(Debug, Subcommand)]
pub enum BackupAction {
    /// Create a new backup snapshot.
    Create {
        /// Notes describing the snapshot.
        #[arg(long)]
        note: Option<String>,

        /// Limit captured references to comma-separated globs.
        #[arg(long)]
        refs: Option<String>,
    },
    /// List existing snapshots.
    List,
    /// Show snapshot details.
    Show {
        /// Snapshot ID.
        snapshot_id: String,
    },
    /// Verify snapshot integrity.
    Verify {
        /// Snapshot ID.
        snapshot_id: String,
    },
    /// Prune old snapshots.
    Prune {
        /// Keep the newest N snapshots.
        #[arg(long)]
        keep: Option<u32>,

        /// Prune snapshots older than the duration (e.g. '30 days').
        #[arg(long)]
        older_than: Option<String>,
    },
}

/// Auth subcommands.
#[derive(Debug, Subcommand)]
pub enum AuthAction {
    /// Add a credential.
    Add {
        /// Hostname (e.g. 'github.com') or repository ID.
        #[arg(long)]
        host: Option<String>,

        /// Authentication method.
        #[arg(long, value_enum)]
        method: Option<AuthMethodArg>,

        /// Username for HTTPS credentials.
        #[arg(long)]
        username: Option<String>,

        /// SSH private key path.
        #[arg(long)]
        key: Option<String>,

        /// Read PAT token / password from stdin.
        #[arg(long)]
        token_stdin: bool,
    },
    /// List stored credentials.
    List,
    /// Remove a credential.
    Remove {
        /// Credential ID.
        id: String,
    },
    /// Test credentials for a host or repository.
    Test {
        /// Hostname or repository ID.
        #[arg(long)]
        host: Option<String>,
    },
}

/// Shared selection & filter flags (CONVENTIONS §9, CLI Spec §5).
#[derive(Debug, Args, Clone)]
pub struct SelectionFlags {
    /// Staleness age threshold (e.g., '1 year ago', '6 months ago').
    #[arg(short, long)]
    pub age: Option<String>,

    /// Select merged branches (ancestors of default branch).
    #[arg(long)]
    pub merged: bool,

    /// Select ONLY unmerged branches.
    #[arg(long)]
    pub unmerged: bool,

    /// Restrict scope to local branches.
    #[arg(long)]
    pub local: bool,

    /// Restrict scope to remote branches.
    #[arg(long)]
    pub remote: bool,

    /// Comma-separated glob/substring patterns to skip (case-insensitive).
    #[arg(short, long)]
    pub exclude: Option<String>,

    /// Extra protected patterns added to the protected set.
    #[arg(long)]
    pub protected: Option<String>,

    /// Restrict to branches that satisfy naming policy.
    #[arg(long)]
    pub standard: bool,

    /// Restrict to branches that violate naming policy.
    #[arg(long)]
    pub non_standard: bool,

    /// Sort ordering (age | name | author | commits | ahead | behind).
    #[arg(long)]
    pub sort: Option<String>,

    /// Free-form policy predicate expression.
    #[arg(long)]
    pub filter: Option<String>,

    /// Cap the number of selected references.
    #[arg(long)]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ActionType {
    Delete,
    Archive,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum MergeStrategyArg {
    Ours,
    Theirs,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum RestoreTarget {
    Local,
    Remote,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ReportType {
    Audit,
    Trend,
    Both,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ReportFormatArg {
    Md,
    Json,
    Html,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum AuthMethodArg {
    Ssh,
    Https,
    Token,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ShellArg {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Elvish,
}
