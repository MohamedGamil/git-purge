# Security Policy

## Reporting a vulnerability

**Please do not report security vulnerabilities through public GitHub issues,
discussions, or pull requests.**

Instead, report them privately using
[GitHub Security Advisories](https://github.com/MohamedGamil/git-purge/security/advisories/new)
("Report a vulnerability"). If you cannot use that, contact the maintainer privately
and we will coordinate a secure channel.

Please include, where possible:

- A description of the issue and its impact.
- Steps to reproduce (a minimal proof of concept if you have one).
- Affected version(s), platform, and configuration.

We aim to acknowledge reports within a few business days and will keep you informed
as we investigate and prepare a fix. We support **coordinated disclosure**: we ask
that you give us a reasonable window to release a fix before any public disclosure,
and we are happy to credit you in the advisory.

## Supported versions

Git Purge is pre-1.0. Security fixes target the latest released minor version and
`main`. Once 1.0 ships, this table will list the supported release lines.

| Version | Supported |
| :--- | :--- |
| `0.1.x` (pre-release) | :white_check_mark: (latest only) |

## Scope & handling notes

Git Purge performs destructive git operations and handles credentials, so we take the
following especially seriously:

- Credential/secret exposure (in logs, errors, snapshots, or reports).
- Any path that performs a destructive operation without the mandated dry-run default
  and pre-operation backup.
- Protected-ref bypasses (deleting `main`/`master`/`HEAD`/etc.).
- Path traversal or writing outside the resolved config/data directories.
