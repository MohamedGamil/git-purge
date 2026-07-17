import { describe, it, expect, beforeEach, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import HistoryView from './HistoryView.vue';
import { useReposStore } from '../stores/repos';
import { useHistoryStore } from '../stores/history';

describe('HistoryView.vue', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  it('should render repository select prompt when no repo is selected', () => {
    const reposStore = useReposStore();
    reposStore.activeRepoId = null;

    const wrapper = mount(HistoryView);
    expect(wrapper.find('h1').text()).toBe('History & Trends');
    expect(wrapper.find('.select-prompt').text()).toContain('Please select a repository');
  });

  it('should render history stats and chart when history data is loaded', async () => {
    const reposStore = useReposStore();
    reposStore.activeRepoId = 'repo-1';
    reposStore.activeRepoDetail = { id: 'repo-1', name: 'Repo One' } as any;

    const historyStore = useHistoryStore();
    vi.spyOn(historyStore, 'fetchHistory').mockResolvedValue([
      { recordedAt: '2026-07-11T00:00:00Z', staleCount: 5, deletedCount: 2, archivedCount: 1 }
    ] as any);
    vi.spyOn(historyStore, 'fetchRuns').mockResolvedValue([] as any);

    const wrapper = mount(HistoryView);
    await new Promise(resolve => setTimeout(resolve));
    await new Promise(resolve => setTimeout(resolve)); // Wait for fetchHistory and fetchRuns to resolve

    expect(wrapper.find('.history-live-dashboard').exists()).toBe(true);
    expect(wrapper.find('.trend-chart').exists()).toBe(true);
    expect(wrapper.text()).toContain('Total Branches Deleted');
  });
});
