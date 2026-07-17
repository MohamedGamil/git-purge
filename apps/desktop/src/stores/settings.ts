import { defineStore } from 'pinia';
import {
  settingsGet,
  settingsSave,
  settingsImport,
  settingsExport,
  type Settings
} from '../api/ipc';
import { useTheme, type ThemeMode } from '../composables/useTheme';

export const useSettingsStore = defineStore('settings', {
  state: () => ({
    settings: null as Settings | null,
    loading: false,
    saving: false,
    error: null as string | null
  }),

  actions: {
    async fetchSettings() {
      this.loading = true;
      this.error = null;
      try {
        const data = await settingsGet();
        this.settings = data;
        
        // Sync theme with local storage & hook
        const { setTheme } = useTheme();
        setTheme(data.theme);
        localStorage.setItem('gitpurge-date-format', data.dateFormat || 'YYYY-MM-DD h:m a');
        return data;
      } catch (err: any) {
        this.error = err?.message || 'Failed to fetch settings';
        throw err;
      } finally {
        this.loading = false;
      }
    },

    async saveSettings(updatedSettings: Settings) {
      this.saving = true;
      this.error = null;
      try {
        const data = await settingsSave(updatedSettings);
        this.settings = data;
        
        // Sync theme
        const { setTheme } = useTheme();
        setTheme(data.theme);
        localStorage.setItem('gitpurge-theme', data.theme);
        localStorage.setItem('gitpurge-date-format', data.dateFormat || 'YYYY-MM-DD h:m a');
        return data;
      } catch (err: any) {
        this.error = err?.message || 'Failed to save settings';
        throw err;
      } finally {
        this.saving = false;
      }
    },

    async importSettings(path: string) {
      this.saving = true;
      this.error = null;
      try {
        const data = await settingsImport(path);
        this.settings = data;
        
        // Sync theme
        const { setTheme } = useTheme();
        setTheme(data.theme);
        localStorage.setItem('gitpurge-theme', data.theme);
        localStorage.setItem('gitpurge-date-format', data.dateFormat || 'YYYY-MM-DD h:m a');
        return data;
      } catch (err: any) {
        this.error = err?.message || 'Failed to import settings';
        throw err;
      } finally {
        this.saving = false;
      }
    },

    async exportSettings(path: string) {
      this.saving = true;
      this.error = null;
      try {
        await settingsExport(path);
      } catch (err: any) {
        this.error = err?.message || 'Failed to export settings';
        throw err;
      } finally {
        this.saving = false;
      }
    }
  }
});
