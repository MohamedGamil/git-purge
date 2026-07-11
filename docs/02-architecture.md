# 02 — Architecture

`Status: Approved` · `Owner: Architecture` · `Last-updated: 2026-07-11` ·
`Related: 01-tech-stack.md, 04-core-spec.md, 15-extensibility.md`

## 1. Guiding principle

> **All behavior lives in `gitpurge-core`. The CLI and the UI are thin, dumb
> adapters that translate user intent into core calls and render core results.**

This is enforced structurally: the CLI crate and the Tauri backend crate both
depend on `gitpurge-core`, and neither is allowed to depend on `gix`, `git2`,
`rusqlite`, etc. directly (checked by an architecture test + `cargo-deny` bans).

## 2. Workspace layout

```
git-purge/
├── Cargo.toml                     # workspace root (members + shared deps)
├── rust-toolchain.toml
├── deny.toml                      # cargo-deny config
├── crates/
│   ├── gitpurge-core/             # ← the brain (lib)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs           # GitPurgeError (thiserror)
│   │       ├── config.rs          # config load/save (TOML)
│   │       ├── model/             # Repo, Branch, Classification, Snapshot, Plan…
│   │       ├── git/               # GitBackend trait + gix/git2/shell impls
│   │       ├── policy/            # age/naming/protected policy engine
│   │       ├── scan/              # classification pipeline
│   │       ├── backup/            # snapshot create/list/verify/restore
│   │       ├── action/            # delete / archive / restore orchestration
│   │       ├── auth/              # credential providers + secure storage
│   │       ├── report/            # report generation + formatters
│   │       ├── history/           # SQLite trend DB
│   │       ├── diff/              # branch/commit diff + tree view
│   │       └── testkit/          # fixture-repo builders (test-only feature)
│   ├── gitpurge-cli/              # bin `git-purge` (clap)
│   │   └── src/{main.rs, cmd/*}
│   └── (gitpurge-desktop lives under apps/desktop/src-tauri)
├── apps/
│   └── desktop/                   # Tauri v2 app
│       ├── src-tauri/             # Rust: gitpurge-desktop bin (commands+events)
│       └── src/                   # Vue 3 frontend
└── .github/workflows/{ci.yml,release.yml}
```

Rationale for `apps/desktop/src-tauri` (vs. a sibling `crates/` member): Tauri
tooling expects the frontend and `src-tauri` colocated; the workspace `Cargo.toml`
still lists it as a member so `cargo build` covers everything.

## 3. Layered design (inside `gitpurge-core`)

```
┌──────────────────────────────────────────────────────────────┐
│ Facade / Services  (scan, backup, action, report, history,    │  ← public API
│                     auth, diff)  — orchestrate use-cases       │
├──────────────────────────────────────────────────────────────┤
│ Policy engine      (age, naming regex, protected, filters)     │
├──────────────────────────────────────────────────────────────┤
│ Domain model       (Repo, Branch, Classification, Snapshot…)   │
├──────────────────────────────────────────────────────────────┤
│ Ports (traits)     GitBackend · SecretStore · HistoryStore ·   │  ← seams for
│                    ReportSink · Clock · ProgressSink           │    extensibility
├──────────────────────────────────────────────────────────────┤
│ Adapters           gix / git2 / shell · keyring · rusqlite ·   │
│                    md/json/html · system clock · event sink    │
└──────────────────────────────────────────────────────────────┘
```

- **Ports & adapters (hexagonal).** Every external concern is a trait ("port")
  with concrete adapters. This is the seam that makes Requirement 6 real: a future
  GitLab provider or a different backup backend is a new adapter, no facade change.
- **Dependency inversion.** Services take ports by generic/`dyn` injection, so tests
  substitute in-memory fakes (deterministic, network-free).

## 4. The shared-core contract

Both adapters call the same service objects:

```rust
// gitpurge-core public surface (illustrative)
pub struct Engine { /* holds injected ports + config */ }

impl Engine {
    pub fn open(config: Config) -> Result<Self, GitPurgeError>;
    pub fn scan(&self, repo: &RepoId, opts: ScanOptions) -> Result<ScanResult>;
    pub fn plan(&self, repo: &RepoId, filter: &ActionFilter) -> Result<Plan>;
    pub fn backup_create(&self, repo: &RepoId, opts: BackupOptions) -> Result<Snapshot>;
    pub fn execute(&self, plan: &Plan, mode: ExecMode) -> Result<RunReport>;
    pub fn restore(&self, snap: &SnapshotId, spec: RestoreSpec) -> Result<RestoreOutcome>;
    pub fn diff(&self, a: &RefSpec, b: &RefSpec) -> Result<DiffResult>;
    pub fn show_tree(&self, at: &RefSpec, path: Option<&Path>) -> Result<TreeView>;
    pub fn report(&self, repo: &RepoId, fmt: ReportFormat) -> Result<Report>;
    pub fn history(&self, repo: &RepoId) -> Result<TrendHistory>;
    // long ops accept a &dyn ProgressSink and a CancellationToken
}
```

- **CLI** parses args → builds `ScanOptions`/`ActionFilter`/`ExecMode` → calls
  `Engine` → renders text/JSON.
- **Tauri backend** exposes each as a `#[tauri::command]` → calls `Engine` →
  returns serde JSON to Vue; forwards `ProgressSink` to Tauri events.

## 5. Concurrency & cancellation
- `Engine` is `Send + Sync`; long operations are `async` (Tokio) and accept a
  `CancellationToken`. The UI can cancel a running scan/backup/delete.
- Progress is reported through a `ProgressSink` port: CLI → `indicatif`; UI →
  Tauri event `gitpurge://progress`.

## 6. Data flow — a delete, end to end (safety-first)
```
1. scan(repo)                → ScanResult (classified branches)
2. plan(repo, filter)        → Plan (what WOULD be deleted; always dry-run first)
3. [user reviews plan in CLI table / UI list]
4. backup_create(pre-op)     → Snapshot (unless --no-backup)  ← Requirement 2/10
5. execute(plan, Execute)    → per-branch:
        delete via GitBackend
        on failure → offer restore from step-4 snapshot   ← auto-restore
6. record RunReport → HistoryStore (SQLite)                ← Requirement 7
7. render report / update trends
```
The same 7 steps run whether invoked from `git-purge delete --execute` or the UI's
"Execute" button, because both call `Engine::backup_create` then `Engine::execute`.

## 7. Desktop process model
- Single Tauri process. Rust side owns `Engine`; the Vue webview never touches git
  directly and has no filesystem/network capability beyond Tauri commands
  (capabilities locked down in `tauri.conf.json`, see [14-security.md](14-security.md)).
- **Standalone-capable:** the UI embeds `gitpurge-core`, so it works with no CLI
  installed. `install-cli`/`git-purge ui` are conveniences, not dependencies
  (Requirement: UI runs even if CLI isn't set up).

## 8. Extensibility seams (detail in [15-extensibility.md](15-extensibility.md))
- New git host (GitLab/Bitbucket): implement a `Provider` port for URL/auth/PR
  metadata; `GitBackend` already abstracts transport.
- New VCS: implement `GitBackend` (or a broader `VcsBackend`) adapter.
- New backup/restore strategy, diff strategy, report format, stats source: each is a
  port with a registry, selected by config.
