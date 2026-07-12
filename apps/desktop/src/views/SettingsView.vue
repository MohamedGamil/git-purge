<template>
  <div class="settings-container">
    <header class="view-header">
      <div>
        <h1>Settings & Policy Manager</h1>
        <p class="subtitle">Configure global behavior, stale branch age thresholds, naming rules, and backups.</p>
      </div>
    </header>

    <div v-if="loading" class="loading-state card">
      <span class="spinner"></span>
      <p>Loading application settings...</p>
    </div>

    <div v-else class="settings-layout">
      <form @submit.prevent="saveSettings" class="settings-form">
        <!-- 1. Design & Appearance -->
        <section class="card settings-section">
          <h3>Appearance</h3>
          <div class="form-group">
            <label for="theme-select">Visual Theme Mode</label>
            <select id="theme-select" v-model="themeMode" @change="handleThemeChange" class="form-input">
              <option value="system">System Default</option>
              <option value="dark">One Dark Pro (Dark)</option>
              <option value="light">One Light (Light)</option>
            </select>
            <p class="field-hint">Theme adjusts automatically when system default is selected.</p>
          </div>
        </section>

        <!-- 2. Clean Policy Configuration -->
        <section class="card settings-section">
          <h3>Staleness & Naming Policies</h3>
          
          <div class="form-group">
            <label for="policy-age">Stale Age Threshold</label>
            <input id="policy-age" type="text" v-model="policyAge" class="form-input" placeholder="e.g. 1 year ago" />
            <p class="field-hint">Branches with last commit older than this threshold are classified as stale.</p>
          </div>

          <div class="form-group">
            <label for="policy-regex">Naming Convention (Allowed Regex)</label>
            <input id="policy-regex" type="text" v-model="policyNamingRegex" class="form-input" placeholder="^(main|master|main-legacy|develop|staging|prod|production|feat/.*|feature/.*|fix/.*|refactor/.*|docs/.*|perf/.*|test/.*|chore/.*|release/.*|hotfix/.*)$" />
            <p class="field-hint">Branches not matching this pattern are classified as non-standard. Leave blank to enforce the default naming convention.</p>
          </div>
        </section>

        <!-- 3. Protected and Excluded references -->
        <section class="card settings-section">
          <h3>Protected & Excluded References</h3>
          
          <div class="form-group">
            <label for="policy-protected">Protected Ref Globs (comma-separated)</label>
            <textarea id="policy-protected" v-model="policyProtected" class="form-input text-area" rows="2" placeholder="main, master, develop, release/*"></textarea>
            <p class="field-hint">Protected references can never be deleted or archived by branch actions (SAFE-03).</p>
          </div>

          <div class="form-group">
            <label for="policy-exclude">Excluded Ref Globs (comma-separated)</label>
            <textarea id="policy-exclude" v-model="policyExclude" class="form-input text-area" rows="2" placeholder="feature/keep-*, experimental/*"></textarea>
            <p class="field-hint">Excluded references are skipped from plan listings and classification scans.</p>
          </div>
        </section>

        <!-- 4. Backup locations -->
        <section class="card settings-section">
          <h3>Backups Configuration</h3>
          <div class="form-group">
            <label for="backups-root">Backups Root Directory</label>
            <div class="input-with-button">
              <input id="backups-root" type="text" v-model="backupsRoot" class="form-input path-input" />
              <button type="button" class="btn btn-secondary btn-sm browse-btn" @click="handleBrowseFolder">
                📁 Browse
              </button>
            </div>
            <p class="field-hint">Base directory where bare mirror repositories are initialized for safe recovery.</p>
          </div>
        </section>

        <!-- Save Button -->
        <div class="actions-bar">
          <button type="submit" class="btn btn-primary btn-save" :disabled="saving">
            <span v-if="saving">Saving Settings...</span>
            <span v-else>💾 Save Changes</span>
          </button>
          <span v-if="saveSuccess" class="success-msg">✓ Settings saved successfully!</span>
        </div>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { useTheme, type ThemeMode } from '../composables/useTheme';
import { settingsGet, settingsSave, type Settings } from '../api/ipc';

const { theme, setTheme } = useTheme();

const loading = ref(true);
const saving = ref(false);
const saveSuccess = ref(false);

// Form Fields State
const themeMode = ref<ThemeMode>(theme.value);
const policyAge = ref('');
const policyNamingRegex = ref('');
const policyProtected = ref('');
const policyExclude = ref('');
const backupsRoot = ref('');

const loadSettings = async () => {
  loading.value = true;
  try {
    const settings = await settingsGet();
    policyAge.value = settings.policy.age;
    policyNamingRegex.value = settings.policy.namingRegex;
    policyProtected.value = settings.policy.protectedRefs.join(', ');
    policyExclude.value = settings.policy.excludeGlobs.join(', ');
    backupsRoot.value = settings.backupsRoot;
    
    // Theme setup
    const savedTheme = localStorage.getItem('gitpurge-theme') as ThemeMode | null;
    if (savedTheme) {
      themeMode.value = savedTheme;
    }
  } catch (err: any) {
    alert('Failed to load settings: ' + err.message);
  } finally {
    loading.value = false;
  }
};

const handleThemeChange = () => {
  setTheme(themeMode.value);
};

const handleBrowseFolder = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Select Backups Directory'
    });
    if (selected && typeof selected === 'string') {
      backupsRoot.value = selected;
    }
  } catch (err: any) {
    alert('Failed to pick directory: ' + err.message);
  }
};

const saveSettings = async () => {
  saving.value = true;
  saveSuccess.value = false;

  const protectedList = policyProtected.value
    .split(',')
    .map(s => s.trim())
    .filter(s => s.length > 0);

  const excludeList = policyExclude.value
    .split(',')
    .map(s => s.trim())
    .filter(s => s.length > 0);

  const settingsPayload: Settings = {
    theme: themeMode.value,
    policy: {
      age: policyAge.value.trim(),
      namingRegex: policyNamingRegex.value.trim(),
      protectedRefs: protectedList,
      excludeGlobs: excludeList
    },
    backupsRoot: backupsRoot.value.trim(),
    defaultNoBackup: false
  };

  try {
    await settingsSave(settingsPayload);
    saveSuccess.value = true;
    setTimeout(() => {
      saveSuccess.value = false;
    }, 3000);
  } catch (err: any) {
    alert('Failed to save settings: ' + err.message);
  } finally {
    saving.value = false;
  }
};

onMounted(() => {
  loadSettings();
});
</script>

<style scoped>
.settings-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  padding: var(--spacing-lg);
  gap: var(--spacing-md);
  overflow-y: auto;
}

.view-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-shrink: 0;
}

.view-header h1 {
  color: var(--on-surface-strong);
  font-size: 24px;
}

.subtitle {
  color: var(--muted);
  font-size: 13px;
}

.loading-state {
  display: flex;
  justify-content: center;
  align-items: center;
  padding: var(--spacing-xl);
  color: var(--muted);
  gap: var(--spacing-sm);
}

.spinner {
  width: 20px;
  height: 20px;
  border: 2px solid var(--border);
  border-top-color: var(--primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.settings-layout {
  display: flex;
  flex-direction: column;
  max-width: 800px;
}

.settings-form {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
}

.settings-section {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
}

.settings-section h3 {
  font-size: 14px;
  color: var(--on-surface-strong);
  border-bottom: 1px solid var(--border);
  padding-bottom: var(--spacing-xs);
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.form-group label {
  font-size: 12px;
  color: var(--on-surface-strong);
  font-weight: 500;
}

.form-input {
  background-color: var(--surface-raised);
  color: var(--on-surface);
  border: 1px solid var(--border);
  padding: var(--spacing-sm);
  border-radius: var(--radius-xs);
  outline: none;
  font-size: 13px;
  width: 100%;
}

.form-input:focus {
  border-color: var(--primary);
}

.text-area {
  font-family: var(--font-mono);
  resize: vertical;
}

.field-hint {
  font-size: 11px;
  color: var(--muted);
  margin-top: 2px;
}

.input-with-button {
  display: flex;
  gap: var(--spacing-sm);
}

.path-input {
  font-family: var(--font-mono);
}

.browse-btn {
  height: 38px;
  white-space: nowrap;
}

.actions-bar {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  padding: var(--spacing-md) 0 var(--spacing-xl);
}

.btn-save {
  padding: var(--spacing-sm) var(--spacing-lg);
  font-weight: 600;
}

.success-msg {
  color: var(--success);
  font-size: 13px;
  font-weight: 500;
  animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateX(-5px); }
  to { opacity: 1; transform: translateX(0); }
}
</style>
