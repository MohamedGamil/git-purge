<template>
  <div class="dashboard-container">
    <header class="dashboard-header">
      <div>
        <h1>Dashboard</h1>
        <p class="subtitle">Welcome to Git Purge. Safely manage your repository branches.</p>
      </div>
      <div class="theme-toggle">
        <label for="theme-select">Theme: </label>
        <select id="theme-select" v-model="currentTheme" @change="handleThemeChange">
          <option value="system">System Default</option>
          <option value="dark">One Dark Pro</option>
          <option value="light">One Light</option>
        </select>
      </div>
    </header>

    <div class="stats-grid">
      <div class="card stat-card">
        <h3>Tracked Repositories</h3>
        <div class="stat-value">{{ repos.length }}</div>
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
        <button class="btn btn-primary" @click="addNewRepo">Add Repository</button>
      </div>

      <div v-if="loading" class="loading-state">
        Loading repositories...
      </div>

      <div v-else-if="repos.length === 0" class="empty-state card">
        <p>No repositories tracked yet. Click "Add Repository" to get started.</p>
      </div>

      <div v-else class="repos-list">
        <div v-for="repo in repos" :key="repo.id" class="repo-item card">
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
      </div>
    </section>

    <section class="ipc-verification card">
      <h2>IPC Connectivity Status</h2>
      <div class="status-indicator" :class="{ 'connected': !loading && !error }">
        <span class="status-dot"></span>
        <span>{{ statusMessage }}</span>
      </div>
      <p v-if="error" class="error-text">Connection Error: {{ error }}</p>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useTheme, type ThemeMode } from '../composables/useTheme';
import { repoList, repoAdd, type RepoSummary } from '../api/ipc';

const { theme, setTheme } = useTheme();
const currentTheme = ref<ThemeMode>(theme.value);

const repos = ref<RepoSummary[]>([]);
const loading = ref(true);
const error = ref<string | null>(null);

const totalBranches = computed(() => repos.value.reduce((acc, r) => acc + r.branchCount, 0));
const totalStale = computed(() => repos.value.reduce((acc, r) => acc + r.stale, 0));

const statusMessage = computed(() => {
  if (loading.value) return 'Connecting to Rust backend...';
  if (error.value) return 'Disconnected from Backend';
  return 'IPC Bridge Active and Connected';
});

const handleThemeChange = () => {
  setTheme(currentTheme.value);
};

const fetchRepos = async () => {
  loading.value = true;
  error.value = null;
  try {
    repos.value = await repoList();
  } catch (err: any) {
    error.value = err?.message || 'Failed to communicate with the Rust backend.';
  } finally {
    loading.value = false;
  }
};

const addNewRepo = async () => {
  try {
    // For now, this is a mock call to add a repo since native pickers require tauri-plugin-dialog
    // This verifies adding repos via IPC works.
    const path = '/home/mgamil/git-purge'; // local path to default project
    await repoAdd(path, undefined, 'Git Purge (Local)');
    await fetchRepos();
  } catch (err: any) {
    alert('Failed to add repository: ' + (err?.message || err));
  }
};

onMounted(() => {
  fetchRepos();
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
}

.repos-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.repo-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.repo-info h3 {
  color: var(--on-surface-strong);
  font-size: 15px;
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

.ipc-verification {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
  margin-top: auto;
}

.ipc-verification h2 {
  font-size: 14px;
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
