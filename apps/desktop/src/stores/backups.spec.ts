import { describe, it, expect, beforeEach, vi, type Mock } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useBackupsStore } from './backups';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

describe('useBackupsStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  it('fetchSnapshots should call backup_list and update state', async () => {
    const mockSnapshots = [
      { id: 'snap-1', repoId: 'repo-1', createdAt: '2026-07-11T00:00:00Z', trigger: 'manual', refCount: 2, verified: true }
    ];
    (invoke as Mock).mockResolvedValue(mockSnapshots);

    const store = useBackupsStore();
    expect(store.snapshots).toEqual([]);
    expect(store.loading).toBe(false);

    const promise = store.fetchSnapshots('repo-1');
    expect(store.loading).toBe(true);

    const res = await promise;
    expect(store.loading).toBe(false);
    expect(store.snapshots).toEqual(mockSnapshots);
    expect(res).toEqual(mockSnapshots);
    expect(invoke).toHaveBeenCalledWith('backup_list', { repoId: 'repo-1' });
  });

  it('fetchSnapshotDetail should call backup_show and populate activeSnapshot', async () => {
    const mockDetail = { id: 'snap-1', repoId: 'repo-1', refs: [] };
    (invoke as Mock).mockResolvedValue(mockDetail);

    const store = useBackupsStore();
    expect(store.activeSnapshot).toBeNull();

    const res = await store.fetchSnapshotDetail('snap-1');
    expect(store.activeSnapshot).toEqual(mockDetail);
    expect(res).toEqual(mockDetail);
    expect(invoke).toHaveBeenCalledWith('backup_show', { snapshotId: 'snap-1' });
  });

  it('verifySnapshot should call backup_verify with progress tracking', async () => {
    let progressCallback: any = null;
    (listen as Mock).mockImplementation((_event, cb) => {
      progressCallback = cb;
      return Promise.resolve(() => {});
    });

    const mockReport = { snapshotId: 'snap-1', ok: true, checkedRefs: 2, problems: [] };
    (invoke as Mock).mockImplementation(async (cmd, args) => {
      if (cmd === 'backup_verify') {
        if (progressCallback) {
          progressCallback({
            payload: {
              taskId: args.taskId,
              phase: 'verify',
              message: 'Verifying snapshot...',
              current: 100,
              total: 100,
              done: true
            }
          });
        }
        return mockReport;
      }
      return Promise.reject(new Error('Unknown cmd'));
    });

    const store = useBackupsStore();
    const promise = store.verifySnapshot('snap-1');
    expect(store.isVerifying).toBe(true);

    const report = await promise;
    expect(store.isVerifying).toBe(false);
    expect(store.verifyReport).toEqual(mockReport);
    expect(store.verifyProgress).toBe(100);
    expect(store.verifyProgressMessage).toBe('Verifying snapshot...');
  });

  it('pruneBackups should trigger backup_prune and fetch updated snapshots list', async () => {
    const mockPruneReport = { removed: ['snap-1'], kept: [], reclaimedBytes: 1024 };
    (invoke as Mock).mockImplementation(async (cmd) => {
      if (cmd === 'backup_prune') return mockPruneReport;
      if (cmd === 'backup_list') return [];
      return Promise.reject(new Error('Unknown cmd'));
    });

    const store = useBackupsStore();
    expect(store.isPruning).toBe(false);

    const report = await store.pruneBackups('repo-1', 0);
    expect(store.isPruning).toBe(false);
    expect(store.pruneReport).toEqual(mockPruneReport);
  });

  it('restoreRef should call restore and manage isRestoring', async () => {
    const mockOutcome = { restored: 'main', as: 'branch', sha: '123' };
    (invoke as Mock).mockResolvedValue(mockOutcome);

    const store = useBackupsStore();
    expect(store.isRestoring).toBe(false);

    const spec = { refName: 'main', targetType: 'branch' as const, force: true };
    const res = await store.restoreRef('snap-1', spec);

    expect(store.isRestoring).toBe(false);
    expect(res).toEqual(mockOutcome);
    expect(invoke).toHaveBeenCalledWith('restore', { snapshotId: 'snap-1', spec });
  });
});
