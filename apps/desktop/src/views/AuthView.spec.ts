import { describe, it, expect, beforeEach, vi, type Mock } from 'vitest';
import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import AuthView from './AuthView.vue';
import { invoke } from '@tauri-apps/api/core';

// Mock Tauri plugin-dialog
vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
}));

// Mock Tauri core invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('AuthView.vue', () => {
  const mockCredentials = [
    { id: '1', host: 'github.com', username: 'git', kind: 'token', provider: 'keyring', meta: { tokenLast4: '1234' } }
  ];

  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
    
    (invoke as Mock).mockImplementation((cmd) => {
      if (cmd === 'auth_list') {
        return Promise.resolve(mockCredentials);
      }
      return Promise.resolve(null);
    });
  });

  it('should render page title and configured credentials', async () => {
    const wrapper = mount(AuthView);
    await new Promise(resolve => setTimeout(resolve));
    
    expect(wrapper.find('h1').text()).toBe('Remote Authentication');
    
    // Verify system default is present
    expect(wrapper.text()).toContain('Default SSH Identity');
    
    // Verify loaded credential
    expect(wrapper.text()).toContain('github.com (git)');
  });
});
