# P3 — Task Cards

> Phase: **3 — CLI adapter** · Status: **Complete** · Est: 10 ED
> Depends on: P1, P2

---

## P3-T1 · Global flags + config resolution

**Goal:** Implement the CLI global flags (`--json`, `--no-color`, `--config`, `--repo`,
`-v/-q`) and config file resolution (XDG/KnownFolders). Wire `Config::resolve()` into
the CLI entry point.

**Files:** `crates/gitpurge-cli/src/main.rs`

**Depends on:** P0-T3

**Acceptance:** `--config <path>` loads custom config; default resolution uses XDG;
`--json` switches output format; `--no-color` / `NO_COLOR` suppresses ANSI.

---

## P3-T2 · repo command group

**Goal:** `repo add/remove/list/set-default` — manage tracked repositories in config.

**Files:** `crates/gitpurge-cli/src/cmd/repo.rs`

**Depends on:** P3-T1

**Acceptance:** `repo add ./path` adds to config; `repo list` shows tracked repos;
`repo remove` drops a repo; `repo set-default` sets the default.

---

## P3-T3 · scan + plan commands

**Goal:** Wire `git-purge scan` → `Engine::scan` and `git-purge plan` → `Engine::plan`.
Human-readable table output and `--json` envelope.

**Files:** `crates/gitpurge-cli/src/cmd/scan.rs`, `crates/gitpurge-cli/src/cmd/plan.rs`

**Depends on:** P1-T7, P3-T1

**Acceptance:** `scan` produces colored branch table matching spec; `plan` shows
"what would happen" with per-action "why" reasons; `--json` wraps in stable envelope.

---

## P3-T4 · delete + archive commands

**Goal:** Wire `git-purge delete` and `git-purge archive` with dry-run default, `--execute`
opt-in, confirmation flow, and backup-before-destroy.

**Files:** `crates/gitpurge-cli/src/cmd/delete.rs`, `crates/gitpurge-cli/src/cmd/archive.rs`

**Depends on:** P2-T7, P3-T3

**Acceptance:** `delete` without `--execute` shows dry-run; `delete --execute` backs up
first, confirms, then deletes; `SAFE-01` and `SAFE-04` proven via CLI.

---

## P3-T5 · backup + restore commands

**Goal:** Wire `git-purge backup create/list/show/verify/prune` and `git-purge restore`.

**Files:** `crates/gitpurge-cli/src/cmd/backup.rs`, `crates/gitpurge-cli/src/cmd/restore.rs`

**Depends on:** P2-T7, P3-T1

**Acceptance:** Full backup lifecycle exercisable from CLI; `restore --as-tag` works.

---

## P3-T6 · diff + show commands

**Goal:** Wire `git-purge diff` → `Engine::diff` and `git-purge show` → `Engine::show_tree`.

**Files:** `crates/gitpurge-cli/src/cmd/diff.rs`, `crates/gitpurge-cli/src/cmd/show.rs`

**Depends on:** P1-T6, P3-T1

**Acceptance:** `diff` shows file-level stats; `show` outputs file content at a ref.

---

## P3-T7 · report + history commands

**Goal:** Wire `git-purge report` and `git-purge history` → `Engine::report/history`.

**Files:** `crates/gitpurge-cli/src/cmd/report.rs`, `crates/gitpurge-cli/src/cmd/history.rs`

**Depends on:** P5 (or stub with todo)

**Acceptance:** `report --format md` emits markdown; `history` shows trend data.

---

## P3-T8 · auth command

**Goal:** Wire `git-purge auth set/get/remove/list` → `Engine` auth methods.

**Files:** `crates/gitpurge-cli/src/cmd/auth.rs`

**Depends on:** P6 (or stub with todo)

**Acceptance:** `auth set` stores credential; `auth get` retrieves; `auth remove` deletes.

---

## P3-T9 · install-cli + completions + ui

**Goal:** `install-cli --user/--system` copies binary to PATH; `completions --shell bash`
emits completions; `ui` launches desktop app if installed.

**Files:** `crates/gitpurge-cli/src/cmd/install.rs`, `crates/gitpurge-cli/src/cmd/completions.rs`

**Depends on:** P3-T1

**Acceptance:** `install-cli --user` places binary on PATH; `completions --shell zsh`
produces valid zsh completions; `ui` with no desktop installed exits gracefully.

---

## P3-T10 · CLI snapshot tests (insta)

**Goal:** `insta` snapshot tests for all commands' human and JSON output on fixture repos.

**Files:** `crates/gitpurge-cli/tests/`

**Depends on:** P3-T2 through P3-T9

**Acceptance:** `cargo insta test` passes; any output format change requires explicit
snapshot review.
