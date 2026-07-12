<template>
  <div class="dashboard-container">
    <header class="dashboard-header">
      <div>
        <h1>Dashboard</h1>
        <p class="subtitle">Safely purge stale branches — with a net under every operation.</p>
      </div>
      <div class="theme-toggle">
        <!-- <label for="theme-select">Theme: </label> -->
        <select id="theme-select" v-model="currentTheme" @change="handleThemeChange">
          <option value="system">System Default Theme</option>
          <option value="dark">Dark Theme</option>
          <option value="light">Light theme</option>
        </select>
      </div>
    </header>

    <div class="stats-grid">
      <div class="card stat-card">
        <h3>Tracked Repositories</h3>
        <div class="stat-value">{{ store.repos.length }}</div>
      </div>
      <div class="card stat-card">
        <h3>Total Branches</h3>
        <div class="stat-value">{{ totalBranches }}</div>
      </div>
      <div class="card stat-card">
        <h3>Stale Branches</h3>
        <div class="stat-value warning-text">{{ totalStale }}</div>
      </div>
    </div>

    <section class="repos-section">
      <div class="section-header">
        <h2>Tracked Repositories</h2>
        <button class="btn btn-primary" @click="handleBrowseFolder" :disabled="store.loading">
          <FolderPlus class="lucide-icon" style="margin-right: 6px;" /> Add Repository
        </button>
      </div>

      <div v-if="store.loading" class="loading-state">
        <span class="spinner"></span> Loading repositories...
      </div>

      <div v-else-if="store.repos.length === 0" class="empty-state card">
        <p>No repositories tracked yet. Click "Add Repository" to select a git folder.</p>
      </div>

      <div v-else class="repos-list">
        <div v-for="repo in store.repos" :key="repo.id" class="repo-item card">
          <div class="repo-main">
            <div class="repo-info">
              <h3>{{ repo.name }}</h3>
              <p class="repo-path" v-if="repo.localPath"><code>{{ repo.localPath }}</code></p>
              <p class="repo-url" v-if="repo.remoteUrl"><code>{{ repo.remoteUrl }}</code></p>
            </div>
            <div class="repo-stats">
              <span class="badge badge-info">{{ repo.branchCount }} Branches</span>
              <span class="badge badge-warning">{{ repo.stale }} Stale</span>
              <span class="badge badge-danger">{{ repo.unmerged }} Unmerged</span>
            </div>
          </div>

          <div class="repo-actions">
            <button class="btn btn-secondary btn-sm" @click="exploreRepo(repo.id)">
              <GitBranch class="lucide-icon" style="margin-right: 4px;" /> Explore
            </button>
            <button class="btn btn-secondary btn-sm" @click="viewBackups(repo.id)">
              <Database class="lucide-icon" style="margin-right: 4px;" /> Backups
            </button>
            <button class="btn btn-danger-alt btn-sm" @click="confirmDeleteId = repo.id">
              <Trash2 class="lucide-icon" style="margin-right: 4px;" /> Remove
            </button>
          </div>

          <!-- Inline Remove Confirmation Dialog (Premium UI) -->
          <div v-if="confirmDeleteId === repo.id" class="remove-confirm-overlay">
            <div class="remove-confirm-box card">
              <h4>Remove Repository?</h4>
              <p>This stops tracking the repository in Git Purge.</p>
              <label class="checkbox-container">
                <input type="checkbox" v-model="dropBackups" />
                <span class="checkmark"></span>
                Also delete all backup snapshots for this repo
              </label>
              <div class="confirm-actions">
                <button class="btn btn-secondary btn-sm" @click="confirmDeleteId = null">Cancel</button>
                <button class="btn btn-danger btn-sm" @click="removeRepo(repo.id)">Confirm Remove</button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>

    <section class="ipc-verification card">
      <h2>IPC Connectivity Status</h2>
      <div class="status-indicator" :class="{ 'connected': !store.loading && !store.error }">
        <span class="status-dot"></span>
        <span>{{ statusMessage }}</span>
      </div>
      <p v-if="store.error" class="error-text">Connection Error: {{ store.error }}</p>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useRouter } from 'vue-router';
import { open } from '@tauri-apps/plugin-dialog';
import { useTheme, type ThemeMode } from '../composables/useTheme';
import { useReposStore } from '../stores/repos';
import { isMock } from '../api/ipc';
import { FolderPlus, GitBranch, Database, Trash2 } from '@lucide/vue';

const router = useRouter();
const store = useReposStore();

const { theme, setTheme } = useTheme();
const currentTheme = ref<ThemeMode>(theme.value);

const confirmDeleteId = ref<string | null>(null);
const dropBackups = ref(false);

const totalBranches = computed(() => store.repos.reduce((acc, r) => acc + r.branchCount, 0));
const totalStale = computed(() => store.repos.reduce((acc, r) => acc + r.stale, 0));

const statusMessage = computed(() => {
  if (isMock) return 'IPC Bridge Active and Connected (Mock Mode)';
  if (store.loading) return 'Connecting to Rust backend...';
  if (store.error) return 'Disconnected from Backend';
  return 'IPC Bridge Active and Connected';
});

const handleThemeChange = () => {
  setTheme(currentTheme.value);
};

const handleBrowseFolder = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Select Git Repository'
    });

    if (selected && typeof selected === 'string') {
      // Parse folder name for the display name
      const name = selected.split(/[/\\]/).pop() || 'Git Repo';
      await store.addRepo(selected, undefined, name);
    }
  } catch (err: any) {
    alert('Failed to pick directory: ' + (err?.message || err));
  }
};

const exploreRepo = async (id: string) => {
  await store.selectRepo(id);
  router.push('/branches');
};

const viewBackups = async (id: string) => {
  store.activeRepoId = id;
  try {
    store.activeRepoDetail = await store.repos.find(r => r.id === id) ? await store.selectRepo(id) as any : null;
  } catch {}
  router.push('/backups');
};

const removeRepo = async (id: string) => {
  try {
    await store.removeRepo(id, dropBackups.value);
    confirmDeleteId.value = null;
    dropBackups.value = false;
  } catch (err: any) {
    alert('Failed to remove repository: ' + err.message);
  }
};

onMounted(() => {
  store.fetchRepos();
});
</script>

<style scoped>
.dashboard-container {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-lg);
  padding: var(--spacing-lg);
  width: 100%;
  height: 100%;
  overflow-y: auto;
}

.dashboard-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.dashboard-header h1 {
  color: var(--on-surface-strong);
  font-weight: 600;
  font-size: 24px;
}

.subtitle {
  color: var(--muted);
  font-size: 14px;
}

.theme-toggle select {
  background-color: var(--surface-raised);
  color: var(--on-surface);
  border: 1px solid var(--border);
  padding: var(--spacing-xs) var(--spacing-sm);
  border-radius: var(--radius-xs);
  outline: none;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: var(--spacing-md);
}

.stat-card {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.stat-card h3 {
  font-size: 12px;
  text-transform: uppercase;
  color: var(--muted);
  letter-spacing: 0.5px;
}

.stat-value {
  font-size: 32px;
  font-weight: 700;
  color: var(--on-surface-strong);
}

.warning-text {
  color: var(--warning);
}

.repos-section {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.section-header h2 {
  color: var(--on-surface-strong);
  font-size: 18px;
}

.loading-state, .empty-state {
  display: flex;
  justify-content: center;
  align-items: center;
  padding: var(--spacing-xl);
  color: var(--muted);
  gap: var(--spacing-sm);
}

.spinner {
  width: 16px;
  height: 16px;
  border: 2px solid var(--border);
  border-top-color: var(--primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.repos-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.repo-item {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
  position: relative;
}

.repo-main {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.repo-info h3 {
  color: var(--on-surface-strong);
  font-size: 16px;
  font-weight: 600;
}

.repo-path, .repo-url {
  font-size: 12px;
  color: var(--muted);
  margin-top: 2px;
}

.repo-stats {
  display: flex;
  gap: var(--spacing-sm);
}

.repo-actions {
  display: flex;
  gap: var(--spacing-sm);
  border-top: 1px solid var(--border);
  padding-top: var(--spacing-sm);
}

.btn-sm {
  padding: var(--spacing-xs) var(--spacing-sm);
  font-size: 12px;
}

.btn-danger-alt {
  background-color: transparent;
  color: var(--danger);
  border: 1px solid rgba(224, 108, 117, 0.3);
}

.btn-danger-alt:hover {
  background-color: rgba(224, 108, 117, 0.1);
  border-color: var(--danger);
}

.icon-spacing {
  margin-right: 4px;
}

.remove-confirm-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: var(--overlay);
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-md);
  z-index: 10;
}

.remove-confirm-box {
  width: 90%;
  max-width: 400px;
  background-color: var(--surface-raised);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
}

.remove-confirm-box h4 {
  color: var(--on-surface-strong);
}

.remove-confirm-box p {
  font-size: 13px;
  color: var(--on-surface);
}

.checkbox-container {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  font-size: 13px;
  color: var(--on-surface);
  cursor: pointer;
}

.confirm-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--spacing-sm);
}

.ipc-verification {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
  margin-top: auto;
}

.ipc-verification h2 {
  font-size: 12px;
  color: var(--muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  font-size: 13px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background-color: var(--danger);
}

.connected .status-dot {
  background-color: var(--success);
  box-shadow: 0 0 8px var(--success);
}

.error-text {
  color: var(--danger);
  font-size: 12px;
}
</style>
