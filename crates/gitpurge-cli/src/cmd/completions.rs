//! Completions command handler (CLI Spec §8.14).

use clap::CommandFactory;
use gitpurge_core::Result;

/// Handle the `completions` command.
pub fn handle_completions(shell: crate::cli::ShellArg) -> Result<()> {
    let mut cmd = crate::cli::Cli::command();
    let generator = match shell {
        crate::cli::ShellArg::Bash => clap_complete::Shell::Bash,
        crate::cli::ShellArg::Zsh => clap_complete::Shell::Zsh,
        crate::cli::ShellArg::Fish => clap_complete::Shell::Fish,
        crate::cli::ShellArg::PowerShell => clap_complete::Shell::PowerShell,
        crate::cli::ShellArg::Elvish => clap_complete::Shell::Elvish,
    };
    clap_complete::generate(generator, &mut cmd, "git-purge", &mut std::io::stdout());
    Ok(())
}
