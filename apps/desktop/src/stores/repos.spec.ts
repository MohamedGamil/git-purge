import { describe, it, expect, beforeEach, vi, type Mock } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useReposStore } from './repos';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// Mock Tauri modules
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

describe('useReposStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  it('fetchRepos should fetch and populate repo list', async () => {
    const mockRepos = [
      { id: '1', name: 'Repo A', branchCount: 3, stale: 1, unmerged: 0, protectedCount: 1 }
    ];
    (invoke as Mock).mockResolvedValue(mockRepos);

    const store = useReposStore();
    expect(store.repos).toEqual([]);
    expect(store.loading).toBe(false);

    const fetchPromise = store.fetchRepos();
    expect(store.loading).toBe(true);

    await fetchPromise;
    expect(store.loading).toBe(false);
    expect(store.repos).toEqual(mockRepos);
    expect(invoke).toHaveBeenCalledWith('repo_list');
  });

  it('addRepo should invoke repo_add and fetch updated list', async () => {
    const mockNewRepo = { id: '2', name: 'Repo B', branchCount: 0, stale: 0, unmerged: 0, protectedCount: 0 };
    (invoke as Mock).mockImplementation((cmd) => {
      if (cmd === 'repo_add') return Promise.resolve(mockNewRepo);
      if (cmd === 'repo_list') return Promise.resolve([mockNewRepo]);
      return Promise.reject(new Error('Unknown command'));
    });

    const store = useReposStore();
    const result = await store.addRepo('/path/to/repo', undefined, 'Repo B');

    expect(invoke).toHaveBeenCalledWith('repo_add', { path: '/path/to/repo', url: undefined, name: 'Repo B' });
    expect(invoke).toHaveBeenCalledWith('repo_list');
    expect(result).toEqual(mockNewRepo);
    expect(store.repos).toEqual([mockNewRepo]);
  });

  it('removeRepo should invoke repo_remove and fetch updated list', async () => {
    (invoke as Mock).mockResolvedValue([]);

    const store = useReposStore();
    store.activeRepoId = '1';
    await store.removeRepo('1', true);

    expect(invoke).toHaveBeenCalledWith('repo_remove', { repoId: '1', dropBackups: true });
    expect(invoke).toHaveBeenCalledWith('repo_list');
    expect(store.activeRepoId).toBeNull();
  });

  it('runScan should trigger scan and update branches state', async () => {
    let progressCallback: any = null;
    (listen as Mock).mockImplementation((_event, cb) => {
      progressCallback = cb;
      return Promise.resolve(() => {}); // return unlisten
    });

    const mockScanResult = {
      repoId: '1',
      scannedAt: '2026-07-11T00:00:00Z',
      branches: [
        { name: 'main', refPath: 'refs/heads/main', tipSha: '123', tipShort: '123', authorName: 'Mohamed', committedAt: '2026-07-11', ageDays: 0, classification: { merge: 'merged', locality: 'local', freshness: 'active', protected: true, naming: 'standard', ahead: 0, behind: 0 } }
      ]
    };

    (invoke as Mock).mockImplementation(async (cmd, args) => {
      if (cmd === 'scan') {
        // Simulate background progress event firing mid-scan
        if (progressCallback) {
          progressCallback({
            payload: {
              taskId: args.taskId,
              phase: 'ref-walk',
              message: 'Walking commits...',
              current: 50,
              total: 100,
              done: false
            }
          });
        }
        return mockScanResult;
      }
      if (cmd === 'repo_show') {
        return { summary: { id: '1' } };
      }
      if (cmd === 'repo_list') {
        return [];
      }
      return Promise.reject(new Error('Unknown command'));
    });

    const store = useReposStore();
    store.activeRepoId = '1';

    const scanPromise = store.runScan('1', { includeRemote: true });
    expect(store.isScanning).toBe(true);

    await scanPromise;
    expect(store.isScanning).toBe(false);
    expect(store.branches).toEqual(mockScanResult.branches);
    expect(store.scannedAt).toBe(mockScanResult.scannedAt);
    expect(store.scanProgress).toBe(50); // should show last event progress before completion
  });

  it('cancelActiveTask should invoke cancel and reset state', async () => {
    (invoke as Mock).mockResolvedValue(null);

    const store = useReposStore();
    store.activeTaskId = 'task-123';
    store.isScanning = true;

    await store.cancelActiveTask();

    expect(invoke).toHaveBeenCalledWith('cancel', { taskId: 'task-123' });
    expect(store.isScanning).toBe(false);
    expect(store.activeTaskId).toBeNull();
  });
});
