//! Architecture guard test (P0-T6, CONVENTIONS §2).
//!
//! Asserts that the CLI crate does not directly depend on git engines
//! (gix, git2), database engines (rusqlite), or keychain libraries (keyring).
//! All such logic must reside inside `gitpurge-core`.

use std::fs;
use std::path::Path;

#[test]
fn test_cli_architecture_guard() {
    let cargo_toml_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let content = fs::read_to_string(cargo_toml_path).expect("Failed to read CLI Cargo.toml");

    let toml: toml::Value = toml::from_str(&content).expect("Failed to parse CLI Cargo.toml");
    let deps = toml
        .get("dependencies")
        .expect("No dependencies section in CLI Cargo.toml");

    let banned = &["gix", "git2", "rusqlite", "keyring"];
    for &ban in banned {
        assert!(
            deps.get(ban).is_none(),
            "CLI crate must not directly depend on '{}' (CONVENTIONS §2). All git/db/keychain logic must live in gitpurge-core.",
            ban
        );
    }
}
