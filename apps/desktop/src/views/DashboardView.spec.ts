import { describe, it, expect, beforeEach, vi, type Mock } from 'vitest';
import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import DashboardView from './DashboardView.vue';
import { useReposStore } from '../stores/repos';
import { invoke } from '@tauri-apps/api/core';

// Mock router
const mockPush = vi.fn();
vi.mock('vue-router', () => ({
  useRouter: () => ({
    push: mockPush,
  }),
}));

// Mock Tauri plugin-dialog
vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
}));

// Mock Tauri core invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('DashboardView.vue', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
    mockPush.mockClear();
    
    // Default invoke mocks
    (invoke as Mock).mockImplementation((cmd) => {
      if (cmd === 'repo_list') {
        return Promise.resolve([
          { id: 'repo-1', name: 'Repo One', branchCount: 10, stale: 3, unmerged: 2, protectedCount: 2, localPath: '/path/1' }
        ]);
      }
      return Promise.resolve(null);
    });
  });

  it('should render page title and subtitle', () => {
    const wrapper = mount(DashboardView);
    expect(wrapper.find('h1').text()).toBe('Dashboard');
    expect(wrapper.find('.subtitle').text()).toContain('Safely purge stale branches');
  });

  it('should compute and display stat values correctly', async () => {
    const wrapper = mount(DashboardView);
    const store = useReposStore();
    
    // Wait for fetchRepos promise
    await new Promise(resolve => setTimeout(resolve));
    
    expect(store.repos.length).toBe(1);
    
    const statCards = wrapper.findAll('.stat-value');
    expect(statCards[0].text()).toBe('1'); // Tracked Repos
    expect(statCards[1].text()).toBe('10'); // Total Branches
    expect(statCards[2].text()).toBe('3'); // Stale Branches
  });

  it('should redirect to branches view when clicking explore', async () => {
    const store = useReposStore();
    vi.spyOn(store, 'selectRepo').mockResolvedValue({} as any);

    const wrapper = mount(DashboardView);
    await new Promise(resolve => setTimeout(resolve));
    
    const exploreBtn = wrapper.find('.repo-actions button');
    expect(exploreBtn.exists()).toBe(true);
    
    await exploreBtn.trigger('click');
    await new Promise(resolve => setTimeout(resolve)); // Wait for async selectRepo promise to resolve
    
    expect(mockPush).toHaveBeenCalledWith('/branches');
  });
});
