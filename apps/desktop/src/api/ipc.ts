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
  autoFetch?: boolean;
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
  targetBranch?: string;
  strategy?: string;
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
    locality: 'local' | 'remote';
    originalRef?: string;
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
  originalRef?: string;
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
  dateFormat: string;
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

// --- Mock API Layer (VITE_MOCK_IPC) ---
export const isMock = (import.meta as any).env?.VITE_MOCK_IPC === 'true' || (typeof (window as any).__TAURI_INTERNALS__ === 'undefined' && (import.meta as any).env?.MODE !== 'test');

let mockRepos: RepoSummary[] = [
  { id: 'git-purge', name: 'git-purge', localPath: '/home/mgamil/git-purge', remoteUrl: 'https://github.com/MohamedGamil/git-purge.git', branchCount: 6, lastScanned: new Date().toISOString(), stale: 2, unmerged: 1, protectedCount: 2 },
  { id: 'react-app', name: 'react-app', localPath: '/home/mgamil/react-app', remoteUrl: 'https://github.com/someuser/react-app.git', branchCount: 15, lastScanned: new Date().toISOString(), stale: 5, unmerged: 2, protectedCount: 1 }
];

let mockRepoDetails: Record<string, RepoDetail> = {
  'git-purge': { id: 'git-purge', name: 'git-purge', localPath: '/home/mgamil/git-purge', remoteUrl: 'https://github.com/MohamedGamil/git-purge.git', branchCount: 6, lastScanned: new Date().toISOString(), stale: 2, unmerged: 1, protectedCount: 2, defaultBranch: 'main', remotes: ['origin'], backupCount: 1 },
  'react-app': { id: 'react-app', name: 'react-app', localPath: '/home/mgamil/react-app', remoteUrl: 'https://github.com/someuser/react-app.git', branchCount: 15, lastScanned: new Date().toISOString(), stale: 5, unmerged: 2, protectedCount: 1, defaultBranch: 'master', remotes: ['origin'], backupCount: 0 }
};

let mockBranches: Record<string, Branch[]> = {
  'git-purge': [
    { name: 'main', refPath: 'refs/heads/main', tipSha: 'a1b2c3d4e5f6', tipShort: 'a1b2c3d', authorName: 'Mohamed Gamil', committedAt: new Date(Date.now() - 2 * 24 * 3600 * 1000).toISOString(), ageDays: 2, classification: { merge: 'merged', locality: 'local', freshness: 'active', protected: true, naming: 'standard', ahead: 0, behind: 0 } },
    { name: 'feature/auth', refPath: 'refs/heads/feature/auth', tipSha: 'b2c3d4e5f6g7', tipShort: 'b2c3d4e', authorName: 'Mohamed Gamil', committedAt: new Date(Date.now() - 100 * 24 * 3600 * 1000).toISOString(), ageDays: 100, classification: { merge: 'merged', locality: 'local', freshness: 'stale', protected: false, naming: 'standard', ahead: 0, behind: 0 } },
    { name: 'feature/dashboard', refPath: 'refs/heads/feature/dashboard', tipSha: 'c3d4e5f6g7h8', tipShort: 'c3d4e5f', authorName: 'John Doe', committedAt: new Date(Date.now() - 400 * 24 * 3600 * 1000).toISOString(), ageDays: 400, classification: { merge: 'unmerged', locality: 'local', freshness: 'stale', protected: false, naming: 'standard', ahead: 2, behind: 1 } },
    { name: 'temp-fix', refPath: 'refs/heads/temp-fix', tipSha: 'd4e5f6g7h8i9', tipShort: 'd4e5f6g', authorName: 'Jane Smith', committedAt: new Date(Date.now() - 15 * 24 * 3600 * 1000).toISOString(), ageDays: 15, classification: { merge: 'merged', locality: 'local', freshness: 'active', protected: false, naming: 'nonStandard', ahead: 0, behind: 0 } },
    { name: 'origin/main', refPath: 'refs/remotes/origin/main', tipSha: 'a1b2c3d4e5f6', tipShort: 'a1b2c3d', authorName: 'Mohamed Gamil', committedAt: new Date(Date.now() - 2 * 24 * 3600 * 1000).toISOString(), ageDays: 2, classification: { merge: 'merged', locality: 'remote', freshness: 'active', protected: true, naming: 'standard', ahead: 0, behind: 0 } },
    { name: 'origin/feature/auth', refPath: 'refs/remotes/origin/feature/auth', tipSha: 'b2c3d4e5f6g7', tipShort: 'b2c3d4e', authorName: 'Mohamed Gamil', committedAt: new Date(Date.now() - 100 * 24 * 3600 * 1000).toISOString(), ageDays: 100, classification: { merge: 'merged', locality: 'remote', freshness: 'stale', protected: false, naming: 'standard', ahead: 0, behind: 0 } }
  ]
};

let mockSnapshots: Record<string, ClientSnapshot[]> = {
  'git-purge': [
    { id: 'snap_01jh234567890abcdef', repoId: 'git-purge', createdAt: new Date(Date.now() - 5 * 24 * 3600 * 1000).toISOString(), trigger: 'preDelete', refCount: 3, verified: true }
  ]
};

let mockSnapshotDetails: Record<string, ClientSnapshotDetail> = {
  'snap_01jh234567890abcdef': {
    id: 'snap_01jh234567890abcdef',
    repoId: 'git-purge',
    createdAt: new Date(Date.now() - 5 * 24 * 3600 * 1000).toISOString(),
    trigger: 'preDelete',
    refCount: 3,
    verified: true,
    refs: [
      { branch: 'feature/auth', tipSha: 'b2c3d4e5f6g7', commitCount: 15, upstream: 'origin/feature/auth', merge: 'merged', locality: 'local' },
      { branch: 'temp-fix', tipSha: 'd4e5f6g7h8i9', commitCount: 2, merge: 'merged', locality: 'local' }
    ]
  }
};

let mockSettings: Settings = {
  theme: 'dark',
  policy: {
    age: '90 days ago',
    namingRegex: '^(main|master|develop|staging|prod|production|feat/.*|feature/.*|fix/.*|refactor/.*|docs/.*|perf/.*|test/.*|chore/.*|release/.*|hotfix/.*)$',
    protectedRefs: ['main', 'master', 'develop', 'production'],
    excludeGlobs: ['dont-touch/*']
  },
  backupsRoot: '/home/mgamil/.git-purge/backups',
  defaultNoBackup: false,
  dateFormat: 'YYYY-MM-DD h:m a'
};

let mockHistory: Record<string, any[]> = {
  'git-purge': [
    { recordedAt: new Date(Date.now() - 30 * 24 * 3600 * 1000).toISOString(), totalBranches: 10, activeCount: 6, staleCount: 4, mergedCount: 8, unmergedCount: 2, deletedCount: 2, archivedCount: 1, nonStandardCount: 1 },
    { recordedAt: new Date(Date.now() - 15 * 24 * 3600 * 1000).toISOString(), totalBranches: 8, activeCount: 6, staleCount: 2, mergedCount: 6, unmergedCount: 2, deletedCount: 1, archivedCount: 0, nonStandardCount: 0 },
    { recordedAt: new Date(Date.now() - 1 * 24 * 3600 * 1000).toISOString(), totalBranches: 7, activeCount: 5, staleCount: 2, mergedCount: 5, unmergedCount: 2, deletedCount: 1, archivedCount: 1, nonStandardCount: 0 }
  ]
};

let mockRuns: Record<string, any[]> = {
  'git-purge': [
    {
      id: 'run-01',
      command: 'delete',
      mode: 'execute',
      startedAt: new Date(Date.now() - 30 * 24 * 3600 * 1000).toISOString(),
      finishedAt: new Date(Date.now() - 30 * 24 * 3600 * 1000 + 5000).toISOString(),
      snapshotId: 'snap-01',
      actor: 'mgamil',
      deletedCount: 2,
      archivedCount: 0,
      branches: ['feature/stale-ui', 'bugfix/typo-main']
    },
    {
      id: 'run-02',
      command: 'archive',
      mode: 'execute',
      startedAt: new Date(Date.now() - 15 * 24 * 3600 * 1000).toISOString(),
      finishedAt: new Date(Date.now() - 15 * 24 * 3600 * 1000 + 4000).toISOString(),
      snapshotId: 'snap-02',
      actor: 'mgamil',
      deletedCount: 0,
      archivedCount: 1,
      branches: ['feature/old-legacy-ref']
    },
    {
      id: 'run-03',
      command: 'delete',
      mode: 'dry-run',
      startedAt: new Date(Date.now() - 1 * 24 * 3600 * 1000).toISOString(),
      finishedAt: new Date(Date.now() - 1 * 24 * 3600 * 1000 + 2000).toISOString(),
      snapshotId: null,
      actor: 'mgamil',
      deletedCount: 1,
      archivedCount: 0,
      branches: ['feature/dry-run-demo']
    }
  ]
};

let mockCredentials: any[] = [
  { id: 'ssh-key-1', label: 'My SSH Key', provider: 'keyring', host: 'github.com', username: 'git', kind: 'ssh', meta: { keyPath: '/home/mgamil/.ssh/id_rsa' } }
];

let progressListeners: ((event: ProgressEvent) => void)[] = [];

export async function repoList(): Promise<RepoSummary[]> {
  if (isMock) {
    return Promise.resolve(JSON.parse(JSON.stringify(mockRepos)));
  }
  return invoke<RepoSummary[]>('repo_list');
}

export async function repoAdd(path?: string, url?: string, name?: string): Promise<RepoSummary> {
  if (isMock) {
    const id = (name || 'repo-' + Math.random().toString(36).slice(2, 7)).toLowerCase();
    const newRepo: RepoSummary = {
      id,
      name: name || 'Mock Repo',
      localPath: path,
      remoteUrl: url,
      branchCount: 3,
      lastScanned: new Date().toISOString(),
      stale: 1,
      unmerged: 1,
      protectedCount: 1
    };
    mockRepos.push(newRepo);
    mockRepoDetails[id] = {
      ...newRepo,
      defaultBranch: 'main',
      remotes: ['origin'],
      backupCount: 0
    };
    mockBranches[id] = [
      { name: 'main', refPath: 'refs/heads/main', tipSha: 'abc123000', tipShort: 'abc1230', authorName: 'Mock Developer', committedAt: new Date().toISOString(), ageDays: 0, classification: { merge: 'merged', locality: 'local', freshness: 'active', protected: true, naming: 'standard', ahead: 0, behind: 0 } },
      { name: 'stale-branch', refPath: 'refs/heads/stale-branch', tipSha: 'xyz987000', tipShort: 'xyz9870', authorName: 'Mock Developer', committedAt: new Date(Date.now() - 120 * 24 * 3600 * 1000).toISOString(), ageDays: 120, classification: { merge: 'merged', locality: 'local', freshness: 'stale', protected: false, naming: 'standard', ahead: 0, behind: 0 } },
      { name: 'unmerged-branch', refPath: 'refs/heads/unmerged-branch', tipSha: 'def456000', tipShort: 'def4560', authorName: 'Mock Developer', committedAt: new Date(Date.now() - 50 * 24 * 3600 * 1000).toISOString(), ageDays: 50, classification: { merge: 'unmerged', locality: 'local', freshness: 'active', protected: false, naming: 'standard', ahead: 1, behind: 0 } }
    ];
    return Promise.resolve(newRepo);
  }
  return invoke<RepoSummary>('repo_add', { path, url, name });
}

export async function repoRemove(repoId: string, dropBackups?: boolean): Promise<void> {
  if (isMock) {
    mockRepos = mockRepos.filter(r => r.id !== repoId);
    delete mockRepoDetails[repoId];
    delete mockBranches[repoId];
    return Promise.resolve();
  }
  return invoke<void>('repo_remove', { repoId, dropBackups });
}

export async function repoShow(repoId: string): Promise<RepoDetail> {
  if (isMock) {
    const detail = mockRepoDetails[repoId];
    if (!detail) return Promise.reject({ code: 'NOT_FOUND', message: 'Repository not found' });
    return Promise.resolve(JSON.parse(JSON.stringify(detail)));
  }
  return invoke<RepoDetail>('repo_show', { repoId });
}

export async function scan(repoId: string, options: ClientScanOptions, taskId: string): Promise<ClientScanResult> {
  if (isMock) {
    const steps = [
      { pct: 10, msg: 'Initializing scan...' },
      { pct: 40, msg: 'Analyzing repository branches...' },
      { pct: 85, msg: 'Resolving protection policies...' },
      { pct: 100, msg: 'Scan complete.' }
    ];
    for (const step of steps) {
      await new Promise(r => setTimeout(r, 200));
      progressListeners.forEach(cb => cb({
        taskId,
        phase: 'scan',
        message: step.msg,
        current: step.pct,
        total: 100,
        done: step.pct === 100
      }));
    }
    const branches = mockBranches[repoId] || [];
    return Promise.resolve({
      repoId,
      scannedAt: new Date().toISOString(),
      branches
    });
  }
  return invoke<ClientScanResult>('scan', { repoId, options, taskId });
}

export async function plan(repoId: string, filter: ClientActionFilter): Promise<ClientPlan> {
  if (isMock) {
    const branches = mockBranches[repoId] || [];
    const actions: ClientPlannedAction[] = [];
    
    const targetBranches = filter.refs && filter.refs.length > 0 
      ? branches.filter(b => filter.refs?.includes(b.name)) 
      : branches.filter(b => b.classification.freshness === 'stale' || b.classification.merge === 'merged');
      
    for (const b of targetBranches) {
      if (b.classification.protected) continue;
      actions.push({
        refName: b.name,
        action: filter.kind,
        reason: b.classification.merge === 'merged' ? 'Branch is merged and unprotected' : 'Branch is stale and unprotected',
        classification: b.classification,
        destructive: b.classification.merge === 'unmerged'
      });
    }
    
    return Promise.resolve({
      repoId,
      kind: filter.kind,
      actions,
      createdAt: new Date().toISOString()
    });
  }
  return invoke<ClientPlan>('plan', { repoId, filter });
}

export async function backupCreate(repoId: string, options: ClientBackupOptions, taskId: string): Promise<ClientSnapshot> {
  if (isMock) {
    const steps = [
      { pct: 10, msg: 'Initializing backup snapshot...' },
      { pct: 50, msg: 'Writing snapshot references to mirror...' },
      { pct: 85, msg: 'Verifying backup integrity...' },
      { pct: 100, msg: 'Backup created successfully.' }
    ];
    for (const step of steps) {
      await new Promise(r => setTimeout(r, 250));
      progressListeners.forEach(cb => cb({
        taskId,
        phase: 'backup',
        message: step.msg,
        current: step.pct,
        total: 100,
        done: step.pct === 100
      }));
    }
    const snapId = 'snap_' + Math.random().toString(36).slice(2, 10);
    const newSnap: ClientSnapshot = {
      id: snapId,
      repoId,
      createdAt: new Date().toISOString(),
      trigger: options.trigger,
      refCount: options.refs?.length || 3,
      verified: true
    };
    if (!mockSnapshots[repoId]) mockSnapshots[repoId] = [];
    mockSnapshots[repoId].push(newSnap);
    
    mockSnapshotDetails[snapId] = {
      id: snapId,
      repoId,
      createdAt: newSnap.createdAt,
      trigger: newSnap.trigger,
      refCount: newSnap.refCount,
      verified: newSnap.verified,
      refs: (options.refs || ['feature/auth', 'temp-fix']).map(ref => ({
        branch: ref,
        tipSha: 'abc' + Math.random().toString(36).slice(2, 6),
        commitCount: 5,
        merge: 'merged',
        locality: ref.startsWith('origin/') ? 'remote' : 'local'
      }))
    };
    
    return Promise.resolve(newSnap);
  }
  return invoke<ClientSnapshot>('backup_create', { repoId, options, taskId });
}

export async function backupList(repoId: string): Promise<ClientSnapshot[]> {
  if (isMock) {
    return Promise.resolve(JSON.parse(JSON.stringify(mockSnapshots[repoId] || [])));
  }
  return invoke<ClientSnapshot[]>('backup_list', { repoId });
}

export async function backupShow(snapshotId: string): Promise<ClientSnapshotDetail> {
  if (isMock) {
    const detail = mockSnapshotDetails[snapshotId];
    if (!detail) return Promise.reject({ code: 'NOT_FOUND', message: 'Snapshot not found' });
    return Promise.resolve(JSON.parse(JSON.stringify(detail)));
  }
  return invoke<ClientSnapshotDetail>('backup_show', { snapshotId });
}

export async function backupVerify(snapshotId: string, taskId: string): Promise<ClientVerifyReport> {
  if (isMock) {
    const steps = [
      { pct: 20, msg: 'Checking snapshot references...' },
      { pct: 60, msg: 'Probing commit objects in mirror...' },
      { pct: 100, msg: 'Verification complete.' }
    ];
    for (const step of steps) {
      await new Promise(r => setTimeout(r, 200));
      progressListeners.forEach(cb => cb({
        taskId,
        phase: 'verify',
        message: step.msg,
        current: step.pct,
        total: 100,
        done: step.pct === 100
      }));
    }
    return Promise.resolve({
      snapshotId,
      ok: true,
      checkedRefs: 2,
      problems: []
    });
  }
  return invoke<ClientVerifyReport>('backup_verify', { snapshotId, taskId });
}

export async function backupPrune(repoId: string, keep?: number, olderThan?: string): Promise<ClientPruneReport> {
  if (isMock) {
    const snaps = mockSnapshots[repoId] || [];
    let removed: string[] = [];
    let kept: string[] = [];
    if (keep !== undefined && snaps.length > keep) {
      const toKeep = snaps.slice(-keep);
      const toRemove = snaps.slice(0, snaps.length - keep);
      removed = toRemove.map(s => s.id);
      kept = toKeep.map(s => s.id);
      mockSnapshots[repoId] = toKeep;
      for (const id of removed) {
        delete mockSnapshotDetails[id];
      }
    } else {
      kept = snaps.map(s => s.id);
    }
    return Promise.resolve({
      removed,
      kept,
      reclaimedBytes: removed.length * 15 * 1024
    });
  }
  return invoke<ClientPruneReport>('backup_prune', { repoId, keep, olderThan });
}

export async function deleteBranches(repoId: string, plan: ClientPlan, exec: ClientExecOptions, taskId: string): Promise<ClientRunReport> {
  if (isMock) {
    const total = plan.actions.length;
    for (let i = 0; i < total; i++) {
      const act = plan.actions[i];
      await new Promise(r => setTimeout(r, 300));
      progressListeners.forEach(cb => cb({
        taskId,
        phase: 'delete',
        message: `Purging branch: ${act.refName}...`,
        current: i + 1,
        total,
        done: i + 1 === total
      }));
    }
    
    if (mockBranches[repoId]) {
      const deletedRefs = plan.actions.map(a => a.refName);
      mockBranches[repoId] = mockBranches[repoId].filter(b => !deletedRefs.includes(b.name));
    }
    
    const snapId = exec.noBackup ? undefined : 'snap_' + Math.random().toString(36).slice(2, 10);
    if (snapId) {
      const newSnap: ClientSnapshot = {
        id: snapId,
        repoId,
        createdAt: new Date().toISOString(),
        trigger: 'preDelete',
        refCount: plan.actions.length,
        verified: true
      };
      if (!mockSnapshots[repoId]) mockSnapshots[repoId] = [];
      mockSnapshots[repoId].push(newSnap);
      mockSnapshotDetails[snapId] = {
        id: snapId,
        repoId,
        createdAt: newSnap.createdAt,
        trigger: newSnap.trigger,
        refCount: newSnap.refCount,
        verified: newSnap.verified,
        refs: plan.actions.map(a => ({
          branch: a.refName,
          tipSha: 'abc123000',
          commitCount: 3,
          merge: a.classification.merge,
          locality: a.classification.locality
        }))
      };
    }
    
    return Promise.resolve({
      runId: 'run_' + Math.random().toString(36).slice(2, 10),
      startedAt: new Date().toISOString(),
      finishedAt: new Date().toISOString(),
      attempted: total,
      succeeded: total,
      failed: 0,
      skipped: 0,
      snapshotId: snapId,
      perRef: plan.actions.map(a => ({
        refName: a.refName,
        outcome: 'done'
      }))
    });
  }
  return invoke<ClientRunReport>('delete_branches', { repoId, plan, exec, taskId });
}

export async function archiveBranches(repoId: string, plan: ClientPlan, exec: ClientExecOptions, taskId: string): Promise<ClientRunReport> {
  if (isMock) {
    const target = exec.targetBranch || 'main-legacy';
    if (mockBranches[repoId] && !mockBranches[repoId].some(b => b.name === target)) {
      mockBranches[repoId].push({
        name: target,
        refPath: `refs/heads/${target}`,
        tipSha: 'abc123000',
        tipShort: 'abc1230',
        authorName: 'Git Purge Archive',
        committedAt: new Date().toISOString(),
        ageDays: 0,
        classification: {
          locality: 'local',
          freshness: 'active',
          merge: 'unmerged',
          protected: true,
          naming: 'standard',
          ahead: 0,
          behind: 0
        }
      });
    }
    return deleteBranches(repoId, plan, exec, taskId);
  }
  return invoke<ClientRunReport>('archive_branches', { repoId, plan, exec, taskId });
}

export async function restore(snapshotId: string, spec: ClientRestoreSpec): Promise<ClientRestoreOutcome> {
  if (isMock) {
    const newName = spec.newName || spec.refName;
    const detail = mockSnapshotDetails[snapshotId];
    if (detail && mockBranches[detail.repoId]) {
      const repoId = detail.repoId;
      const matchedRef = detail.refs.find(r => r.branch === spec.refName);
      if (matchedRef) {
        const alreadyExists = mockBranches[repoId].some(b => b.name === newName);
        if (alreadyExists && !spec.force) {
          return Promise.reject({ code: 'SAFETY', message: `Branch ${newName} already exists. Use force to overwrite.` });
        }
        mockBranches[repoId] = mockBranches[repoId].filter(b => b.name !== newName);
        mockBranches[repoId].push({
          name: newName,
          refPath: `refs/heads/${newName}`,
          tipSha: matchedRef.tipSha,
          tipShort: matchedRef.tipSha.slice(0, 7),
          authorName: 'Restored from Backup',
          committedAt: new Date().toISOString(),
          ageDays: 0,
          classification: {
            merge: matchedRef.merge,
            locality: 'local',
            freshness: 'active',
            protected: false,
            naming: 'standard',
            ahead: 0,
            behind: 0
          }
        });
      }
    }
    return Promise.resolve({
      restored: newName,
      as: spec.targetType,
      sha: 'abc123000'
    });
  }
  return invoke<ClientRestoreOutcome>('restore', { snapshotId, spec });
}

export async function diff(repoId: string, a: string, b: string): Promise<ClientDiffResult> {
  if (isMock) {
    return Promise.resolve({
      a: { repoId, ref: a },
      b: { repoId, ref: b },
      files: [
        { path: 'src/main.rs', status: 'modified', added: 10, removed: 5 },
        { path: 'Cargo.toml', status: 'modified', added: 1, removed: 1 },
        { path: 'src/new_helper.rs', status: 'added', added: 45, removed: 0 }
      ],
      ahead: 3,
      behind: 1
    });
  }
  return invoke<ClientDiffResult>('diff', { repoId, a, b });
}

export async function showTree(repoId: string, at: string, path?: string): Promise<ClientTreeView> {
  if (isMock) {
    const entries: ClientTreeView['entries'] = path 
      ? [
          { name: 'main.rs', path: 'src/main.rs', kind: 'file', size: 1024, mode: '100644' },
          { name: 'helper.rs', path: 'src/helper.rs', kind: 'file', size: 512, mode: '100644' }
        ]
      : [
          { name: 'src', path: 'src', kind: 'dir', mode: '040000' },
          { name: 'Cargo.toml', path: 'Cargo.toml', kind: 'file', size: 234, mode: '100644' }
        ];
    return Promise.resolve({
      at: { repoId, ref: at },
      path: path || '',
      entries,
      blob: path ? { text: `// Mock file content for ${path}\nfn main() {\n    println!("Hello Git Purge!");\n}`, truncated: false, binary: false } : undefined
    });
  }
  return invoke<ClientTreeView>('show_tree', { repoId, at, path });
}

export async function settingsGet(): Promise<Settings> {
  if (isMock) {
    return Promise.resolve(JSON.parse(JSON.stringify(mockSettings)));
  }
  return invoke<Settings>('settings_get');
}

export async function settingsSave(settings: Settings): Promise<Settings> {
  if (isMock) {
    mockSettings = JSON.parse(JSON.stringify(settings));
    return Promise.resolve(mockSettings);
  }
  return invoke<Settings>('settings_save', { settings });
}

export async function settingsExport(path: string): Promise<void> {
  if (isMock) {
    console.log(`Mock exporting settings to ${path}`);
    return Promise.resolve();
  }
  return invoke<void>('settings_export', { path });
}

export async function settingsImport(path: string): Promise<Settings> {
  if (isMock) {
    console.log(`Mock importing settings from ${path}`);
    return Promise.resolve(mockSettings);
  }
  return invoke<Settings>('settings_import', { path });
}

export async function cancel(taskId: string): Promise<void> {
  if (isMock) {
    console.log(`Mock cancellation of task ${taskId}`);
    return Promise.resolve();
  }
  return invoke<void>('cancel', { taskId });
}

export async function saveFile(path: string, content: string): Promise<void> {
  if (isMock) {
    console.log(`Mock saving file to ${path} with length ${content.length}`);
    return Promise.resolve();
  }
  return invoke<void>('save_file', { path, content });
}

export async function historyGet(repoId: string): Promise<any> {
  if (isMock) {
    return Promise.resolve(JSON.parse(JSON.stringify(mockHistory[repoId] || [])));
  }
  return invoke<any>('history_get', { repoId });
}

export async function historyRunsGet(repoId: string, limit: number, offset: number): Promise<any> {
  if (isMock) {
    const runs = mockRuns[repoId] || [];
    return Promise.resolve(JSON.parse(JSON.stringify(runs.slice(offset, offset + limit))));
  }
  return invoke<any>('history_runs_get', { repoId, limit, offset });
}

export async function reportGenerate(repoId: string, format: string, reportType?: string): Promise<any> {
  if (isMock) {
    const repo = mockRepos.find(r => r.id === repoId) || mockRepos[0];
    const dateStr = new Date().toLocaleString();
    let content = '';
    
    if (reportType === 'trend') {
      if (format === 'json') {
        content = JSON.stringify({
          reportType: 'trend',
          repoName: repo.name,
          generatedAt: dateStr,
          milestones: ['Stale branches reduced by 10% since baseline'],
          history: mockHistory[repoId] || []
        }, null, 2);
      } else if (format === 'html') {
        content = `<html><body><h1>Trend Report for ${repo.name}</h1><p>Generated at ${dateStr}</p></body></html>`;
      } else {
        content = `### 🔄 Compare against Previous Run\nComparing current state to run on **Fri Jul 10 15:02:26 2026 +0300**:\n\n| Metric | Old Value | New Value | Absolute Change | Change Ratio (%) |\n| :--- | :---: | :---: | :---: | :---: |\n| **Total Branches** | ${repo.branchCount + 2} | ${repo.branchCount} | **-2** | **-25.0%** |\n| **Stale Branches** | ${repo.stale + 1} | ${repo.stale} | **-1** | **-33.3%** |\n\n> [!TIP]\n> **Cleanup Milestone**: Stale branches have been reduced by **1** branch since the baseline run!\n\n### 📜 Run History Log\n| Run Date | Total | Active | Stale | Merged | Unmerged |\n| :--- | :---: | :---: | :---: | :---: | :---: |\n| ${dateStr} | ${repo.branchCount} | ${repo.branchCount - repo.stale} | ${repo.stale} | ${repo.branchCount - repo.unmerged} | ${repo.unmerged} |`;
      }
    } else {
      if (format === 'json') {
        content = JSON.stringify({
          reportType: 'audit',
          repoName: repo.name,
          localPath: repo.localPath,
          generatedAt: dateStr,
          metrics: { total: repo.branchCount, stale: repo.stale, unmerged: repo.unmerged, protected: repo.protectedCount }
        }, null, 2);
      } else if (format === 'html') {
        content = `<html><body><h1>Audit Report for ${repo.name}</h1><p>Generated at ${dateStr}</p></body></html>`;
      } else {
        content = `# Audit Report for ${repo.name}\n\nGenerated at ${dateStr}\n\n- **Total Branches:** ${repo.branchCount}\n- **Stale Branches:** ${repo.stale}\n- **Unmerged Branches:** ${repo.unmerged}`;
      }
    }
    return Promise.resolve({
      content,
      generatedAt: new Date().toISOString()
    });
  }
  return invoke<any>('report_generate', { repoId, format, reportType });
}

export async function authAdd(credential: any): Promise<any> {
  if (isMock) {
    const cred = {
      ...credential,
      id: credential.id || 'auth-' + Math.random().toString(36).slice(2, 7)
    };
    mockCredentials.push(cred);
    return Promise.resolve(cred);
  }
  return invoke<any>('auth_add', { credential });
}

export async function authList(): Promise<any> {
  if (isMock) {
    return Promise.resolve(JSON.parse(JSON.stringify(mockCredentials)));
  }
  return invoke<any>('auth_list');
}

export async function authRemove(id: string): Promise<void> {
  if (isMock) {
    mockCredentials = mockCredentials.filter(c => c.id !== id);
    return Promise.resolve();
  }
  return invoke<void>('auth_remove', { id });
}

export async function authTest(id: string): Promise<boolean> {
  if (isMock) {
    return new Promise(r => setTimeout(() => r(true), 1000));
  }
  return invoke<boolean>('auth_test', { id });
}

// --- Progress Event Listener ---

export function listenProgress(callback: (event: ProgressEvent) => void): Promise<UnlistenFn> {
  if (isMock) {
    progressListeners.push(callback);
    const unlisten: UnlistenFn = () => {
      progressListeners = progressListeners.filter(cb => cb !== callback);
    };
    return Promise.resolve(unlisten);
  }
  return listen<ProgressEvent>('gitpurge://progress', (event) => {
    callback(event.payload);
  });
}
