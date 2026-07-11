//! Confirmation prompts for interactive and script environments (CLI Spec §4.2).

use dialoguer::console::Term;

/// Returns true if the user confirmed standard operation, false otherwise.
pub fn confirm_standard(prompt: &str, yes_flag: bool) -> bool {
    if yes_flag {
        return true;
    }
    if !Term::stdout().is_term() {
        // Non-interactive/non-TTY: refuse by default if yes_flag is false
        return false;
    }
    dialoguer::Confirm::new()
        .with_prompt(prompt)
        .default(false)
        .interact()
        .unwrap_or(false)
}

/// Returns true if the user typed the repo_id correctly, false otherwise.
pub fn confirm_strong(repo_id: &str, prompt: &str, force_flag: bool) -> bool {
    if force_flag {
        return true;
    }
    if !Term::stdout().is_term() {
        return false;
    }
    println!("{}", prompt);
    let entered: String = dialoguer::Input::new()
        .with_prompt(format!("Type the repo id '{}' to confirm", repo_id))
        .interact_text()
        .unwrap_or_default();
    entered.trim() == repo_id
}
