//! Exit codes and error mapping (CLI Spec §7, CONVENTIONS §11).

use gitpurge_core::GitPurgeError;
use miette::Diagnostic;

pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_GENERIC: i32 = 1;
pub const EXIT_USAGE: i32 = 2;
pub const EXIT_CONFIG: i32 = 3;
pub const EXIT_NOT_FOUND: i32 = 4;
pub const EXIT_GIT: i32 = 5;
pub const EXIT_AUTH: i32 = 6;
pub const EXIT_SAFETY: i32 = 7;
pub const EXIT_BACKUP: i32 = 8;
pub const EXIT_PARTIAL: i32 = 9;
pub const EXIT_CANCELLED: i32 = 10;

#[derive(Debug, Diagnostic)]
#[diagnostic(code(cli::error), help("{hint}"))]
pub struct CliError {
    pub message: String,
    pub hint: String,
    pub code: i32,
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CliError {}

pub fn handle_error(err: gitpurge_core::GitPurgeError) -> ! {
    let (code, msg, hint) = match err {
        GitPurgeError::Config(m) => (EXIT_CONFIG, m, "Check your config.toml layout or CLI override path.".to_string()),
        GitPurgeError::RepoNotFound(m) | GitPurgeError::RefNotFound(m) => {
            (EXIT_NOT_FOUND, m, "Verify that the repository is registered and the reference exists.".to_string())
        }
        GitPurgeError::Git(m) | GitPurgeError::BackendUnsupported(m) => {
            (EXIT_GIT, m, "Ensure the repository is not corrupted and remote access is available.".to_string())
        }
        GitPurgeError::Auth(m) => {
            (EXIT_AUTH, m, "Check your credentials or SSH keys/agents.".to_string())
        }
        GitPurgeError::SafetyViolation(m) => {
            (EXIT_SAFETY, m, "Branch is protected, has unmerged commits, or is current HEAD.".to_string())
        }
        GitPurgeError::Snapshot(m) => {
            (EXIT_BACKUP, m, "A backup snapshot failed integrity checks. Mutations aborted.".to_string())
        }
        GitPurgeError::History(m) => {
            (EXIT_PARTIAL, m, "History store error occurred.".to_string())
        }
        GitPurgeError::Io(e) => {
            (EXIT_GENERIC, e.to_string(), "Check file and directory permissions.".to_string())
        }
        GitPurgeError::Other(m) => {
            (EXIT_GENERIC, m, "An unexpected internal error occurred.".to_string())
        }
        _ => {
            (EXIT_GENERIC, "An unexpected error occurred.".to_string(), "Verify logs and configuration.".to_string())
        }
    };

    eprintln!("Error: {}", msg);
    if !hint.is_empty() {
        eprintln!("Help: {}", hint);
    }
    std::process::exit(code);
}
