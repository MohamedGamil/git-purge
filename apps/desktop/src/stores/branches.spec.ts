import { describe, it, expect, beforeEach, vi, type Mock } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useBranchesStore } from './branches';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

describe('useBranchesStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();

    // Default mock implementation
    (invoke as Mock).mockImplementation((cmd) => {
      if (cmd === 'get_active_cleanups') {
        return Promise.resolve([]);
      }
      return Promise.resolve(null);
    });
  });

  it('generatePlan should call plan API and populate planResult', async () => {
    const mockPlan = {
      repoId: 'repo-1',
      kind: 'delete',
      actions: [{ refName: 'stale-branch', action: 'delete', reason: 'stale', destructive: false, classification: {} }],
      createdAt: '2026-07-11'
    };
    (invoke as Mock).mockResolvedValue(mockPlan);

    const store = useBranchesStore();
    expect(store.planResult).toBeNull();
    expect(store.loadingPlan).toBe(false);

    const promise = store.generatePlan('repo-1', { kind: 'delete' });
    expect(store.loadingPlan).toBe(true);

    const res = await promise;
    expect(store.loadingPlan).toBe(false);
    expect(store.planResult).toEqual(mockPlan);
    expect(res).toEqual(mockPlan);
    expect(invoke).toHaveBeenCalledWith('plan', { repoId: 'repo-1', filter: { kind: 'delete' } });
  });

  it('executeDelete should trigger delete and listen to progress', async () => {
    let progressCallback: any = null;
    (listen as Mock).mockImplementation((_event, cb) => {
      progressCallback = cb;
      return Promise.resolve(() => {});
    });

    const mockReport = {
      runId: 'run-1',
      startedAt: '2026-07-11',
      finishedAt: '2026-07-11',
      attempted: 1,
      succeeded: 1,
      failed: 0,
      skipped: 0,
      perRef: []
    };

    (invoke as Mock).mockImplementation(async (cmd, args) => {
      if (cmd === 'delete_branches') {
        if (progressCallback) {
          progressCallback({
            payload: {
              taskId: args.taskId,
              phase: 'delete',
              message: 'Deleting branch...',
              current: 1,
              total: 1,
              done: true
            }
          });
        }
        return mockReport;
      }
      if (cmd === 'get_active_cleanups') {
        return [];
      }
      return Promise.reject(new Error('Unknown cmd'));
    });

    const store = useBranchesStore();
    const planData = { repoId: 'repo-1', kind: 'delete', actions: [], createdAt: '2026-07-11' };
    const execPromise = store.executeDelete('repo-1', planData, { noBackup: true });

    expect(store.isExecuting).toBe(true);
    await execPromise;
    expect(store.isExecuting).toBe(false);
    expect(store.runReport).toEqual(mockReport);
    expect(store.execProgress).toBe(100);
    expect(store.execProgressMessage).toBe('Deleting branch...');
  });

  it('cancelActiveTask should invoke cancel and reset state', async () => {
    (invoke as Mock).mockResolvedValue(null);

    const store = useBranchesStore();
    store.execTaskId = 'task-123';
    store.isExecuting = true;

    await store.cancelActiveTask();

    expect(invoke).toHaveBeenCalledWith('cancel', { taskId: 'task-123' });
    expect(store.isExecuting).toBe(false);
    expect(store.execTaskId).toBeNull();
  });
});
