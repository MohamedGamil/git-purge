-- schema.sql — Git Purge History DDL (CONVENTIONS §5, doc 10 §3.2)

-- One row per tracked repository.
CREATE TABLE IF NOT EXISTS repos (
    id             TEXT PRIMARY KEY,
    canonical_url  TEXT,
    local_path     TEXT,
    path_hash      TEXT NOT NULL,
    display_name   TEXT NOT NULL,
    default_branch TEXT NOT NULL DEFAULT 'origin/main',
    created_at     TEXT NOT NULL,
    tombstoned_at  TEXT
);

-- One row per recorded execution.
CREATE TABLE IF NOT EXISTS runs (
    id            TEXT PRIMARY KEY,
    repo_id       TEXT NOT NULL REFERENCES repos(id) ON DELETE CASCADE,
    command       TEXT NOT NULL,
    mode          TEXT NOT NULL,
    started_at    TEXT NOT NULL,
    finished_at   TEXT,
    snapshot_id   TEXT,
    age_threshold TEXT,
    actor         TEXT,
    tool_version  TEXT NOT NULL,
    exit_code     INTEGER,
    note          TEXT,
    UNIQUE (repo_id, started_at, command)
);

-- Repo-level metrics timeseries (one row per run).
CREATE TABLE IF NOT EXISTS metrics (
    run_id        TEXT PRIMARY KEY REFERENCES runs(id) ON DELETE CASCADE,
    repo_id       TEXT NOT NULL REFERENCES repos(id) ON DELETE CASCADE,
    captured_at   TEXT NOT NULL,
    total         INTEGER NOT NULL,
    active        INTEGER NOT NULL,
    stale         INTEGER NOT NULL,
    merged        INTEGER NOT NULL,
    unmerged      INTEGER NOT NULL,
    non_standard  INTEGER NOT NULL,
    local_count   INTEGER,
    remote_count  INTEGER,
    protected     INTEGER,
    deleted       INTEGER,
    archived      INTEGER,
    restored      INTEGER,
    metrics_hash  TEXT NOT NULL
);

-- Per-branch snapshot rows.
CREATE TABLE IF NOT EXISTS branch_snapshots (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id          TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
    repo_id         TEXT NOT NULL REFERENCES repos(id) ON DELETE CASCADE,
    ref_name        TEXT NOT NULL,
    kind            TEXT NOT NULL,
    tip_sha         TEXT NOT NULL,
    commit_count    INTEGER,
    last_commit_at  TEXT NOT NULL,
    author_name     TEXT,
    author_email    TEXT,
    subject         TEXT,
    upstream        TEXT,
    ahead           INTEGER,
    behind          INTEGER,
    is_merged       INTEGER NOT NULL,
    is_stale        INTEGER NOT NULL,
    is_protected    INTEGER NOT NULL,
    is_standard     INTEGER NOT NULL,
    violation_reason TEXT
);

-- Backup snapshots.
CREATE TABLE IF NOT EXISTS snapshots (
    id           TEXT PRIMARY KEY,
    repo_id      TEXT NOT NULL REFERENCES repos(id) ON DELETE CASCADE,
    created_at   TEXT NOT NULL,
    trigger      TEXT NOT NULL,
    ref_count    INTEGER NOT NULL,
    note         TEXT,
    verified_at  TEXT,
    manifest_ref TEXT NOT NULL,
    backup_path  TEXT
);

CREATE INDEX IF NOT EXISTS idx_runs_repo_time    ON runs(repo_id, started_at);
CREATE INDEX IF NOT EXISTS idx_metrics_repo_time ON metrics(repo_id, captured_at);
CREATE INDEX IF NOT EXISTS idx_bsnap_run         ON branch_snapshots(run_id);
CREATE INDEX IF NOT EXISTS idx_bsnap_repo_ref    ON branch_snapshots(repo_id, ref_name);
