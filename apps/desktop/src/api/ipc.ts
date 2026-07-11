import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

// --- Type Definitions (docs/06 §2.2) ---

export interface SerializableError {
  code: string;
  message: string;
  hint?: string;
}

export interface RepoSummary {
  id: string;
  name: string;
  localPath?: string;
  remoteUrl?: string;
  branchCount: number;
  lastScanned?: string;
  stale: number;
  unmerged: number;
  protectedCount: number;
}

export interface RepoDetail {
  id: string;
  name: string;
  localPath?: string;
  remoteUrl?: string;
  branchCount: number;
  lastScanned?: string;
  stale: number;
  unmerged: number;
  protectedCount: number;
  defaultBranch: string;
  remotes: string[];
  backupCount: number;
}

export interface ClientScanOptions {
  age?: string;
  naming?: boolean;
  includeRemote?: boolean;
}

export interface Classification {
  merge: 'merged' | 'unmerged';
  locality: 'local' | 'remote';
  freshness: 'stale' | 'active';
  protected: boolean;
  naming: 'standard' | 'nonStandard';
  ahead: number;
  behind: number;
}

export interface Branch {
  name: string;
  refPath: string;
  tipSha: string;
  tipShort: string;
  authorName: string;
  committedAt: string;
  ageDays: number;
  upstream?: string;
  classification: Classification;
}

export interface ClientScanResult {
  repoId: string;
  scannedAt: string;
  branches: Branch[];
}

export interface ClientActionFilter {
  kind: 'delete' | 'archive';
  age?: string;
  merged?: boolean;
  includeUnmerged?: boolean;
  exclude?: string[];
  refs?: string[];
}

export interface ClientPlannedAction {
  refName: string;
  action: 'delete' | 'archive';
  reason: string;
  classification: Classification;
  destructive: boolean;
}

export interface ClientPlan {
  repoId: string;
  kind: string;
  actions: ClientPlannedAction[];
  createdAt: string;
}

export interface ClientExecOptions {
  noBackup: boolean;
  confirmedToken?: string;
}

export interface ClientRunReport {
  runId: string;
  startedAt: string;
  finishedAt: string;
  attempted: number;
  succeeded: number;
  failed: number;
  skipped: number;
  snapshotId?: string;
  perRef: {
    refName: string;
    outcome: 'done' | 'failed' | 'skipped' | 'restored';
    error?: SerializableError;
  }[];
}

export interface ClientBackupOptions {
  trigger: 'manual' | 'preDelete' | 'scheduled';
  refs?: string[];
  verify: boolean;
}

export interface ClientSnapshot {
  id: string;
  repoId: string;
  createdAt: string;
  trigger: 'manual' | 'preDelete' | 'scheduled';
  refCount: number;
  verified: boolean;
}

export interface ClientSnapshotDetail {
  id: string;
  repoId: string;
  createdAt: string;
  trigger: 'manual' | 'preDelete' | 'scheduled';
  refCount: number;
  verified: boolean;
  refs: {
    branch: string;
    tipSha: string;
    commitCount: number;
    upstream?: string;
    merge: 'merged' | 'unmerged';
  }[];
}

export interface ClientVerifyReport {
  snapshotId: string;
  ok: boolean;
  checkedRefs: number;
  problems: string[];
}

export interface ClientPruneReport {
  removed: string[];
  kept: string[];
  reclaimedBytes: number;
}

export interface ClientRestoreSpec {
  refName: string;
  targetType: 'branch' | 'tag';
  newName?: string;
  force: boolean;
}

export interface ClientRestoreOutcome {
  restored: string;
  as: 'branch' | 'tag';
  sha: string;
}

export interface ClientDiffResult {
  a: { repoId: string; ref: string };
  b: { repoId: string; ref: string };
  files: {
    path: string;
    status: 'added' | 'modified' | 'deleted' | 'renamed';
    added: number;
    removed: number;
  }[];
  ahead: number;
  behind: number;
}

export interface ClientTreeView {
  at: { repoId: string; ref: string };
  path: string;
  entries: {
    name: string;
    path: string;
    kind: 'dir' | 'file';
    size?: number;
    mode: string;
  }[];
  blob?: {
    text: string;
    truncated: boolean;
    binary: boolean;
  };
}

export interface Settings {
  theme: 'light' | 'dark' | 'system';
  policy: {
    age: string;
    namingRegex: string;
    protectedRefs: string[];
    excludeGlobs: string[];
  };
  backupsRoot: string;
  defaultNoBackup: boolean;
}

export interface ProgressEvent {
  taskId: string;
  phase: string;
  message: string;
  current: number;
  total: number;
  done: boolean;
  error?: SerializableError;
}

// --- IPC Commands ---

export async function repoList(): Promise<RepoSummary[]> {
  return invoke<RepoSummary[]>('repo_list');
}

export async function repoAdd(path?: string, url?: string, name?: string): Promise<RepoSummary> {
  return invoke<RepoSummary>('repo_add', { path, url, name });
}

export async function repoRemove(repoId: string, dropBackups?: boolean): Promise<void> {
  return invoke<void>('repo_remove', { repoId, dropBackups });
}

export async function repoShow(repoId: string): Promise<RepoDetail> {
  return invoke<RepoDetail>('repo_show', { repoId });
}

export async function scan(repoId: string, options: ClientScanOptions, taskId: string): Promise<ClientScanResult> {
  return invoke<ClientScanResult>('scan', { repoId, options, taskId });
}

export async function plan(repoId: string, filter: ClientActionFilter): Promise<ClientPlan> {
  return invoke<ClientPlan>('plan', { repoId, filter });
}

export async function backupCreate(repoId: string, options: ClientBackupOptions, taskId: string): Promise<ClientSnapshot> {
  return invoke<ClientSnapshot>('backup_create', { repoId, options, taskId });
}

export async function backupList(repoId: string): Promise<ClientSnapshot[]> {
  return invoke<ClientSnapshot[]>('backup_list', { repoId });
}

export async function backupShow(snapshotId: string): Promise<ClientSnapshotDetail> {
  return invoke<ClientSnapshotDetail>('backup_show', { snapshotId });
}

export async function backupVerify(snapshotId: string, taskId: string): Promise<ClientVerifyReport> {
  return invoke<ClientVerifyReport>('backup_verify', { snapshotId, taskId });
}

export async function backupPrune(repoId: string, keep?: number, olderThan?: string): Promise<ClientPruneReport> {
  return invoke<ClientPruneReport>('backup_prune', { repoId, keep, olderThan });
}

export async function deleteBranches(repoId: string, plan: ClientPlan, exec: ClientExecOptions, taskId: string): Promise<ClientRunReport> {
  return invoke<ClientRunReport>('delete_branches', { repoId, plan, exec, taskId });
}

export async function archiveBranches(repoId: string, plan: ClientPlan, exec: ClientExecOptions, taskId: string): Promise<ClientRunReport> {
  return invoke<ClientRunReport>('archive_branches', { repoId, plan, exec, taskId });
}

export async function restore(snapshotId: string, spec: ClientRestoreSpec): Promise<ClientRestoreOutcome> {
  return invoke<ClientRestoreOutcome>('restore', { snapshotId, spec });
}

export async function diff(repoId: string, a: string, b: string): Promise<ClientDiffResult> {
  return invoke<ClientDiffResult>('diff', { repoId, a, b });
}

export async function showTree(repoId: string, at: string, path?: string): Promise<ClientTreeView> {
  return invoke<ClientTreeView>('show_tree', { repoId, at, path });
}

export async function settingsGet(): Promise<Settings> {
  return invoke<Settings>('settings_get');
}

export async function settingsSave(settings: Settings): Promise<Settings> {
  return invoke<Settings>('settings_save', { settings });
}

export async function cancel(taskId: string): Promise<void> {
  return invoke<void>('cancel', { taskId });
}

// --- Progress Event Listener ---

export function listenProgress(callback: (event: ProgressEvent) => void): Promise<UnlistenFn> {
  return listen<ProgressEvent>('gitpurge://progress', (event) => {
    callback(event.payload);
  });
}
