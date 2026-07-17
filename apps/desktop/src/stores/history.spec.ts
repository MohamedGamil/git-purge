import { describe, it, expect, beforeEach, vi, type Mock } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useHistoryStore } from './history';
import { invoke } from '@tauri-apps/api/core';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('useHistoryStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  it('fetchHistory should call history_get and populate historyData', async () => {
    const mockHistory = [
      { recordedAt: '2026-07-11', totalBranches: 5, activeCount: 3, staleCount: 2, mergedCount: 4, unmergedCount: 1, deletedCount: 0, archivedCount: 0, nonStandardCount: 0 }
    ];
    (invoke as Mock).mockResolvedValue(mockHistory);

    const store = useHistoryStore();
    expect(store.historyData).toEqual([]);
    expect(store.loading).toBe(false);

    const promise = store.fetchHistory('repo-1');
    expect(store.loading).toBe(true);

    const res = await promise;
    expect(store.loading).toBe(false);
    expect(store.historyData).toEqual(mockHistory);
    expect(res).toEqual(mockHistory);
    expect(invoke).toHaveBeenCalledWith('history_get', { repoId: 'repo-1' });
  });

  it('fetchRuns should call history_runs_get and manage runs list', async () => {
    const mockRuns = [{ id: 'run-1', command: 'delete' }];
    (invoke as Mock).mockResolvedValue(mockRuns);

    const store = useHistoryStore();
    expect(store.runs).toEqual([]);

    await store.fetchRuns('repo-1', 10, 0);
    expect(store.runs).toEqual(mockRuns);
    expect(invoke).toHaveBeenCalledWith('history_runs_get', { repoId: 'repo-1', limit: 10, offset: 0 });

    const extraRuns = [{ id: 'run-2', command: 'archive' }];
    (invoke as Mock).mockResolvedValue(extraRuns);
    await store.fetchRuns('repo-1', 10, 10);
    expect(store.runs).toEqual([...mockRuns, ...extraRuns]);
    expect(invoke).toHaveBeenCalledWith('history_runs_get', { repoId: 'repo-1', limit: 10, offset: 10 });
  });

  it('generateReport should call report_generate and populate reportContent', async () => {
    const mockReport = { content: '# Audit Report', generatedAt: '2026-07-11' };
    (invoke as Mock).mockResolvedValue(mockReport);

    const store = useHistoryStore();
    expect(store.reportContent).toBe('');
    expect(store.isGeneratingReport).toBe(false);

    const res = await store.generateReport('repo-1', 'md', 'audit');
    expect(store.isGeneratingReport).toBe(false);
    expect(store.reportContent).toBe('# Audit Report');
    expect(res).toEqual(mockReport);
    expect(invoke).toHaveBeenCalledWith('report_generate', { repoId: 'repo-1', format: 'md', reportType: 'audit' });
  });
});
