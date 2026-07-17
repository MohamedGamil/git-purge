# Security Policy & Threat Model

Git Purge takes codebase safety and credential security seriously. This document outlines our threat model, security architecture, hardening mitigations, and how we handle security disclosures.

---

## 1. Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues, discussions, or pull requests.**

Instead, report them privately using [GitHub Security Advisories](https://github.com/MohamedGamil/git-purge/security/advisories/new) ("Report a vulnerability"). If you cannot use that, contact the maintainer privately and we will coordinate a secure channel.

Please include, where possible:
- A description of the issue and its impact.
- Steps to reproduce (a minimal proof of concept if you have one).
- Affected version(s), platform, and configuration.

We aim to acknowledge reports within a few business days and will keep you informed as we investigate and prepare a fix. We support **coordinated disclosure**: we ask that you give us a reasonable window to release a fix before any public disclosure, and we are happy to credit you in the advisory.

---

## 2. Supported Versions

Git Purge is in active pre-release. Security fixes target the latest released minor version and the `main` branch.

| Version | Supported |
| :--- | :--- |
| `0.3.x` (current) | :white_check_mark: (latest only) |

---

## 3. Threat Model (STRIDE-lite)

### 3.1 Assets We Protect

| Asset | Where it lives | Why it matters |
| :--- | :--- | :--- |
| **Repository data** (branches, commits, working history) | User's local repos + remotes | Primary user value; destructive ops can lose unmerged work. |
| **Credentials** (SSH passphrases, HTTPS passwords, PAT tokens) | OS keychain / encrypted-file vault | Compromise leads to remote repository access. |
| **Backups / snapshots** | `<data_dir>/git-purge/backups/*.git` | The safety net; if corrupted, restore fails. |
| **History DB & logs** | `<data_dir>/git-purge/history.db`, logs | Audit trail; must contain zero secrets or credential details. |
| **Configuration** | `<config_dir>/git-purge/config.toml` | Integrity of protected-ref list & naming policy. |

### 3.2 Trust Boundaries

- **Webview ↔ Rust Engine:** The Vue webview is treated as untrusted. It can only call the explicit set of `#[tauri::command]`s. It has no access to shell-out, ambient filesystem APIs, or direct network access.
- **Process ↔ Network Remotes:** Authenticated via SSH keys or HTTPS tokens. Host authenticity is verified using standard SSH/TLS.
- **Process ↔ Filesystem/Keychain:** Scoped to the directories resolved by the OS and keychains authenticated via standard keychain APIs.

### 3.3 STRIDE Mitigations

- **Spoofing:** Host verification is strictly enforced during transport. `auth test` allows validating resolved hosts.
- **Tampering:** Supply-chain security is checked via `cargo-deny` and `cargo-audit` in CI. Released binaries are accompanied by SHA-256 checksums and minisign signatures.
- **Tampering (Webview):** The Tauri webview is sandboxed with a strict Content Security Policy (CSP), disabled remote content, prototype freezing, and disabled asset protocols.
- **Repudiation:** Every mutation is logged to an append-only audit journal on disk, and executions are recorded in the SQLite history database.
- **Information Disclosure:** **SAFE-07** enforces secret hygiene. Secrets are stored zeroized in memory, redacted in `Debug` output, and never written to logs, reports, or snapshot metadata.
- **Denial of Service:** Memory and thread usage is bounded. Streaming ref enumeration avoids loading huge trees in memory.
- **Elevation of Privilege:** No `sh -c` or command shells are spawned. We call libraries (`gix`/`git2`) directly. Any CLI fallback spawns argv-only child processes.

---

## 4. Hardening Safeguards & Safety Invariants

We verify every build against the following **7 Safety Invariants (SAFE-01 to SAFE-07)** using dedicated regression suites:

*   **SAFE-01 (Dry-run default):** Every mutating CLI command or UI operation defaults to dry-run (plan only). Applying mutations requires explicit `--execute` flags or user confirmation.
*   **SAFE-02 (Protected references):** Well-known branches (`main`, `master`, `develop`, `staging`, `production`, `HEAD`) and user-defined globs/lists are structurally protected and can never be deleted or archived.
*   **SAFE-03 (Tags preservation):** Tags are strictly preserved and never deleted by branch operations.
*   **SAFE-04 (Pre-op backups):** A verified pre-op snapshot is taken before any delete/archive operation unless `--no-backup` is explicitly declared.
*   **SAFE-05 (Auto-restore on failure):** Any failed deletion or archiving offers the user the option to restore from the pre-operation snapshot.
*   **SAFE-06 (No-force restore):** Restore operations never force-overwrite an existing reference without explicit, separate consent.
*   **SAFE-07 (Secret hygiene):** Zero secret material (tokens, passwords, keys) is written to logs, console output, snapshots, or metrics databases. Secret store structs override `Debug` to print redacted content only.
