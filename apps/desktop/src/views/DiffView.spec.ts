import { describe, it, expect, beforeEach, vi } from 'vitest';
import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import DiffView from './DiffView.vue';
import { useReposStore } from '../stores/repos';

// Mock router
vi.mock('vue-router', () => ({
  useRouter: () => ({
    push: vi.fn(),
  }),
  useRoute: () => ({
    query: { branchA: '', branchB: '' }
  })
}));

describe('DiffView.vue', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  it('should render repository select prompt when no repo is selected', () => {
    const reposStore = useReposStore();
    reposStore.activeRepoId = null;

    const wrapper = mount(DiffView);
    expect(wrapper.find('h1').text()).toBe('Compare & Diff');
    expect(wrapper.find('.select-prompt').text()).toContain('Please select a repository');
  });

  it('should render branch selectors when repo is selected', async () => {
    const reposStore = useReposStore();
    reposStore.activeRepoId = 'repo-1';
    reposStore.activeRepoDetail = { id: 'repo-1', name: 'Repo One' } as any;
    reposStore.branches = [
      { name: 'main', classification: { locality: 'local' } } as any,
      { name: 'feature-1', classification: { locality: 'local' } } as any
    ];

    const wrapper = mount(DiffView);
    await new Promise(resolve => setTimeout(resolve));

    expect(wrapper.find('#branch-a').exists()).toBe(true);
    expect(wrapper.find('#branch-b').exists()).toBe(true);
  });
});
