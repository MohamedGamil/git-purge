import { describe, it, expect, beforeEach, vi, type Mock } from 'vitest';
import { mount } from '@vue/test-utils';
import { createPinia, setActivePinia } from 'pinia';
import SettingsView from './SettingsView.vue';
import { invoke } from '@tauri-apps/api/core';

// Mock Tauri plugin-dialog
vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
  save: vi.fn(),
}));

// Mock Tauri core invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

// Mock useTheme
vi.mock('../composables/useTheme', () => ({
  useTheme: () => ({
    theme: { value: 'dark' },
    setTheme: vi.fn(),
  }),
}));

describe('SettingsView.vue', () => {
  const mockSettings = {
    theme: 'dark',
    policy: {
      age: '90 days ago',
      namingRegex: '^(main|master)$',
      protectedRefs: ['main', 'master'],
      excludeGlobs: ['temp/*'],
    },
    backupsRoot: '/backups',
    defaultNoBackup: false,
    dateFormat: 'locale',
  };

  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
    
    (invoke as Mock).mockImplementation((cmd) => {
      if (cmd === 'settings_get') {
        return Promise.resolve(mockSettings);
      }
      return Promise.resolve(null);
    });
  });

  it('should render form with loaded settings values', async () => {
    const wrapper = mount(SettingsView);
    await new Promise(resolve => setTimeout(resolve));
    
    expect(wrapper.find('h1').text()).toBe('Settings & Policy Manager');
    
    const themeSelect = wrapper.find('#theme-select');
    expect(themeSelect.exists()).toBe(true);
    expect((themeSelect.element as HTMLSelectElement).value).toBe('dark');
    
    const policyAgeInput = wrapper.find('#policy-age');
    expect(policyAgeInput.exists()).toBe(true);
    expect((policyAgeInput.element as HTMLInputElement).value).toBe('90 days ago');
  });

  it('should validate naming regex pattern on input and display error', async () => {
    const wrapper = mount(SettingsView);
    await new Promise(resolve => setTimeout(resolve));
    
    const regexInput = wrapper.find('#policy-regex');
    expect(regexInput.exists()).toBe(true);
    
    // Set invalid regex value
    await regexInput.setValue('['); // Malformed bracket group
    
    expect(wrapper.find('.validation-error-msg').exists()).toBe(true);
    expect(wrapper.find('.validation-error-msg').text()).toContain('character class');
  });
});
