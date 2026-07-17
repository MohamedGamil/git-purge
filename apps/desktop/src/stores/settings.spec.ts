import { describe, it, expect, beforeEach, vi, type Mock } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useSettingsStore } from './settings';
import { invoke } from '@tauri-apps/api/core';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('../composables/useTheme', () => ({
  useTheme: () => ({
    theme: { value: 'dark' },
    setTheme: vi.fn(),
  }),
}));

describe('useSettingsStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.clearAllMocks();
  });

  const mockSettingsData = {
    theme: 'dark' as const,
    policy: {
      age: '90 days ago',
      namingRegex: '.*',
      protectedRefs: ['main'],
      excludeGlobs: []
    },
    backupsRoot: '/backups',
    defaultNoBackup: false,
    dateFormat: 'YYYY-MM-DD'
  };

  it('fetchSettings should call settings_get and populate settings state', async () => {
    (invoke as Mock).mockResolvedValue(mockSettingsData);

    const store = useSettingsStore();
    expect(store.settings).toBeNull();

    await store.fetchSettings();
    expect(store.settings).toEqual(mockSettingsData);
    expect(invoke).toHaveBeenCalledWith('settings_get');
  });

  it('saveSettings should call settings_save and update settings state', async () => {
    (invoke as Mock).mockResolvedValue(mockSettingsData);

    const store = useSettingsStore();
    await store.saveSettings(mockSettingsData);
    expect(store.settings).toEqual(mockSettingsData);
    expect(invoke).toHaveBeenCalledWith('settings_save', { settings: mockSettingsData });
  });

  it('importSettings should call settings_import and update settings state', async () => {
    (invoke as Mock).mockResolvedValue(mockSettingsData);

    const store = useSettingsStore();
    await store.importSettings('/path/to/import.toml');
    expect(store.settings).toEqual(mockSettingsData);
    expect(invoke).toHaveBeenCalledWith('settings_import', { path: '/path/to/import.toml' });
  });

  it('exportSettings should call settings_export', async () => {
    (invoke as Mock).mockResolvedValue(null);

    const store = useSettingsStore();
    await store.exportSettings('/path/to/export.toml');
    expect(invoke).toHaveBeenCalledWith('settings_export', { path: '/path/to/export.toml' });
  });
});
