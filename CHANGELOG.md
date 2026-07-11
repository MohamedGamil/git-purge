# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0-beta.1] — TBD

### Added

- **Core library** (`gitpurge-core`): hexagonal architecture with domain model,
  port traits (GitBackend, SecretStore, HistoryStore, ReportSink, Clock,
  ProgressSink), and Engine facade with full API surface (scan, plan, backup,
  execute, restore, diff, show_tree, report, history).
- **Backup & Restore Subsystem** (Phase 2):
  - Completed `verify.rs` for snapshot manifest, reference tip, and commit reachability validation.
  - Completed `prune.rs` supporting `RetentionPolicy` rules and physical resource reclaiming.
  - Completed `restore.rs` implementing local ref fetching back to source repositories with `SAFE-06` guardrails.
  - Completed `guard.rs` facilitating automatic pre-op backup (`SAFE-04`) and callback-driven auto-restore on mutation failure (`SAFE-05`).
- **CLI Commands** (Phase 3):
  - Clap skeleton supporting global flags, XDG config routing, human-friendly tables, and raw `--json` envelopes.
  - Verbs: `repo add/list/remove/set-default`, `scan`, `plan`, `delete`, `archive`, `backup`, `restore`, `diff`, `show`.
  - Stubs: `report`, `history`, `auth`, `ui`, `completions`, `install-cli`.
  - TTY-aware confirmation prompts for standard/strong mutation gates.
- **Desktop stub** (`gitpurge-desktop`): Tauri v2 workspace member (UI lands in
  Phase 4).
- **Domain model**: Repository, Branch, Commit, Tag, Ref, Classification, Policy,
  Snapshot, Plan, Action, RunReport, Config, and all supporting value objects.
- **Port trait fakes**: FakeGitBackend, FakeSecretStore, FakeHistoryStore,
  FakeReportSink, FakeClock, FakeProgressSink — proving dependency inversion.
- **Configuration**: TOML-based config with XDG/KnownFolders resolution.
- **Error model**: `GitPurgeError` enum with `miette` diagnostic integration.
- **CI**: GitHub Actions workflow (fmt, clippy, nextest, cargo-deny).
- **Project scaffolding**: workspace layout, delivery phases, conventions,
  architecture spec, safety model, 16 specification documents.

### Changed

- MSRV bumped from 1.82 → 1.88 (required by `time`, `home`, `icu` crate
  ecosystem).
- License changed from dual MIT/Apache-2.0 to Apache-2.0.

[Unreleased]: https://github.com/MohamedGamil/git-purge/compare/v0.1.0-beta.1...HEAD
[0.1.0-beta.1]: https://github.com/MohamedGamil/git-purge/releases/tag/v0.1.0-beta.1
