import { describe, it, expect, beforeEach, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import BackupsView from './BackupsView.vue';
import { useReposStore } from '../stores/repos';
import { useBackupsStore } from '../stores/backups';

describe('BackupsView.vue', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  it('should render repo selection prompt if no repository is active', () => {
    const reposStore = useReposStore();
    reposStore.activeRepoId = null;

    const wrapper = mount(BackupsView);
    expect(wrapper.find('h1').text()).toBe('Backups & Restore Points');
    expect(wrapper.find('.select-prompt').text()).toContain('Please select a repository');
  });

  it('should render snapshots list when repository is selected', async () => {
    const reposStore = useReposStore();
    reposStore.activeRepoId = 'repo-1';
    reposStore.activeRepoDetail = { id: 'repo-1', name: 'Repo One' } as any;

    const backupsStore = useBackupsStore();
    backupsStore.snapshots = [
      { id: 'snap-1', repoId: 'repo-1', createdAt: '2026-07-11T00:00:00Z', trigger: 'manual', refCount: 3, verified: true }
    ];

    const wrapper = mount(BackupsView);
    await new Promise(resolve => setTimeout(resolve));

    expect(wrapper.find('.snapshots-list').exists()).toBe(true);
    expect(wrapper.find('.snap-id').text()).toContain('snap-1');
  });
});
