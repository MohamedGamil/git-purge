import { describe, it, expect, beforeEach, vi, type Mock } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import PlanView from './PlanView.vue';
import { useReposStore } from '../stores/repos';
import { invoke } from '@tauri-apps/api/core';

// Mock router
const mockPush = vi.fn();
vi.mock('vue-router', () => ({
  useRouter: () => ({
    push: mockPush,
  }),
  useRoute: () => ({
    query: { repoId: 'repo-1', actionKind: 'delete', refs: 'b1,b2' }
  })
}));

// Mock Tauri core invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('PlanView.vue', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
    mockPush.mockClear();
  });

  it('should render proposed actions list from planResult', async () => {
    const reposStore = useReposStore();
    reposStore.activeRepoDetail = { id: 'repo-1', name: 'Repo One' } as any;

    const mockPlan = {
      repoId: 'repo-1',
      kind: 'delete',
      actions: [
        {
          refName: 'stale-branch',
          action: 'delete',
          reason: 'Stale branch',
          destructive: false,
          classification: { locality: 'local', freshness: 'stale', merge: 'merged' } as any
        }
      ],
      createdAt: '2026-07-11'
    };

    (invoke as Mock).mockImplementation((cmd) => {
      if (cmd === 'get_active_cleanups') {
        return Promise.resolve([]);
      }
      if (cmd === 'plan') {
        return Promise.resolve(mockPlan);
      }
      return Promise.resolve(null);
    });

    const wrapper = mount(PlanView);
    await flushPromises();

    expect(wrapper.find('h1').text()).toBe('Action Plan Review');
    expect(wrapper.find('.action-branch').text()).toContain('stale-branch');
    expect(wrapper.find('.action-reason').text()).toContain('Stale branch');
  });
});
