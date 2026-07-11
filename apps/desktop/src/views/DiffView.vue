<template>
  <div class="diff-container">
    <header class="view-header">
      <div>
        <h1>Compare & Diff</h1>
        <p class="subtitle" v-if="store.activeRepoDetail">
          Comparing references in <strong>{{ store.activeRepoDetail.name }}</strong>
        </p>
      </div>

      <div class="repo-selector">
        <label for="repo-select">Repository: </label>
        <select id="repo-select" v-model="selectedRepoId" @change="handleRepoChange">
          <option value="" disabled>-- Select Repository --</option>
          <option v-for="repo in store.repos" :key="repo.id" :value="repo.id">
            {{ repo.name }}
          </option>
        </select>
      </div>
    </header>

    <div v-if="!store.activeRepoId" class="select-prompt card">
      <p>Please select a repository from the dropdown above to use the Compare tool.</p>
    </div>

    <div v-else class="diff-layout">
      <!-- Selector bar -->
      <section class="selector-card card">
        <div class="branch-select-box">
          <div class="select-item">
            <label for="branch-a">Base Reference (Branch A)</label>
            <input
              type="text"
              v-model="searchA"
              placeholder="Search branch..."
              class="form-input search-input-sub"
            />
            <select id="branch-a" v-model="branchA" class="form-input">
              <option value="" disabled>-- Select Base Ref --</option>
              <option v-for="b in filteredBranchesA" :key="'a-' + b.name" :value="b.name">
                {{ b.name }}
              </option>
            </select>
          </div>

          <button
            class="compare-icon-btn"
            @click="swapBranches"
            title="Swap references A and B"
            :disabled="!branchA && !branchB"
          >
            ⇆
          </button>

          <div class="select-item">
            <label for="branch-b">Compare Reference (Branch B)</label>
            <input
              type="text"
              v-model="searchB"
              placeholder="Search branch..."
              class="form-input search-input-sub"
            />
            <select id="branch-b" v-model="branchB" class="form-input">
              <option value="" disabled>-- Select Compare Ref --</option>
              <option v-for="b in filteredBranchesB" :key="'b-' + b.name" :value="b.name">
                {{ b.name }}
              </option>
            </select>
          </div>
        </div>

        <button class="btn btn-primary compare-btn" @click="runDiff" :disabled="!branchA || !branchB || loading">
          <span v-if="loading">Comparing...</span>
          <span v-else>🔍 Compare References</span>
        </button>
      </section>

      <!-- Diff results area -->
      <section v-if="diffResult" class="results-card card">
        <div class="stats-summary">
          <div class="summary-item">
            <span class="lbl">Commits Ahead (B vs A)</span>
            <span class="val text-success">{{ diffResult.ahead }}</span>
          </div>
          <div class="summary-item">
            <span class="lbl">Commits Behind</span>
            <span class="val text-danger">{{ diffResult.behind }}</span>
          </div>
          <div class="summary-item">
            <span class="lbl">Total Files Changed</span>
            <span class="val">{{ diffResult.files.length }}</span>
          </div>
        </div>

        <div class="files-list-wrapper">
          <h3>Changed Files</h3>
          <div v-if="diffResult.files.length === 0" class="no-changes">
            No file modifications detected between these references. They are synchronized.
          </div>
          <table v-else class="files-table">
            <thead>
              <tr>
                <th>Status</th>
                <th>File Path</th>
                <th class="text-right">Additions</th>
                <th class="text-right">Deletions</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="file in diffResult.files" :key="file.path">
                <td>
                  <span class="badge" :class="statusBadgeClass(file.status)">
                    {{ file.status }}
                  </span>
                </td>
                <td class="file-path"><code>{{ file.path }}</code></td>
                <td class="text-right text-success">+{{ file.added }}</td>
                <td class="text-right text-danger">-{{ file.removed }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import { useRoute } from 'vue-router';
import { useReposStore } from '../stores/repos';
import { diff, type ClientDiffResult } from '../api/ipc';

const route = useRoute();
const store = useReposStore();

const selectedRepoId = ref(store.activeRepoId || '');
const branchA = ref('');
const branchB = ref('');
const searchA = ref('');
const searchB = ref('');
const loading = ref(false);
const diffResult = ref<ClientDiffResult | null>(null);

const filteredBranchesA = computed(() => {
  const q = searchA.value.toLowerCase().trim();
  if (!q) return store.branches;
  return store.branches.filter(b => b.name.toLowerCase().includes(q));
});

const filteredBranchesB = computed(() => {
  const q = searchB.value.toLowerCase().trim();
  if (!q) return store.branches;
  return store.branches.filter(b => b.name.toLowerCase().includes(q));
});

const swapBranches = () => {
  const temp = branchA.value;
  branchA.value = branchB.value;
  branchB.value = temp;
  if (branchA.value && branchB.value) {
    runDiff();
  }
};

const handleRepoChange = () => {
  if (selectedRepoId.value) {
    store.selectRepo(selectedRepoId.value);
    branchA.value = '';
    branchB.value = '';
    diffResult.value = null;
  }
};

const runDiff = async () => {
  if (!selectedRepoId.value || !branchA.value || !branchB.value) return;
  loading.value = true;
  diffResult.value = null;
  try {
    diffResult.value = await diff(selectedRepoId.value, branchA.value, branchB.value);
  } catch (err: any) {
    alert('Diff failed: ' + err.message);
  } finally {
    loading.value = false;
  }
};

const statusBadgeClass = (status: string) => {
  if (status === 'added') return 'badge-success';
  if (status === 'deleted') return 'badge-danger';
  if (status === 'renamed') return 'badge-purple';
  return 'badge-warning';
};

watch(() => store.activeRepoId, (newId) => {
  if (newId) {
    selectedRepoId.value = newId;
    diffResult.value = null;
  }
});

onMounted(() => {
  if (route.query.branchA && route.query.branchB) {
    branchA.value = route.query.branchA as string;
    branchB.value = route.query.branchB as string;
    runDiff();
  }
});
</script>

<style scoped>
.diff-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  padding: var(--spacing-lg);
  gap: var(--spacing-md);
  overflow: hidden;
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

.repo-selector select {
  background-color: var(--surface-raised);
  color: var(--on-surface);
  border: 1px solid var(--border);
  padding: var(--spacing-xs) var(--spacing-sm);
  border-radius: var(--radius-xs);
  outline: none;
}

.select-prompt {
  display: flex;
  justify-content: center;
  align-items: center;
  flex-grow: 1;
  color: var(--muted);
}

.diff-layout {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
  flex-grow: 1;
  overflow: hidden;
}

/* Selector Card */
.selector-card {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  flex-shrink: 0;
  gap: var(--spacing-md);
}

.branch-select-box {
  display: flex;
  align-items: flex-end;
  gap: var(--spacing-md);
  flex-grow: 1;
}

.select-item {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
  flex-grow: 1;
}

.select-item label {
  font-size: 12px;
  color: var(--on-surface);
}

.compare-icon-btn {
  font-size: 20px;
  background-color: var(--surface-raised);
  color: var(--on-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-xs);
  padding: 0 var(--spacing-sm);
  cursor: pointer;
  height: 38px;
  min-width: 38px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all var(--transition-fast);
}

.compare-icon-btn:hover:not(:disabled) {
  background-color: var(--border);
  color: var(--primary);
}

.compare-icon-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.search-input-sub {
  margin-bottom: var(--spacing-xs);
  padding: 6px var(--spacing-sm);
  font-size: 12px;
  height: 32px;
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

.compare-btn {
  height: 38px;
}

/* Results Card */
.results-card {
  flex-grow: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.stats-summary {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--spacing-md);
  border-bottom: 1px solid var(--border);
  padding-bottom: var(--spacing-md);
  margin-bottom: var(--spacing-md);
  flex-shrink: 0;
}

.summary-item {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.summary-item .lbl {
  font-size: 11px;
  color: var(--muted);
  text-transform: uppercase;
}

.summary-item .val {
  font-size: 24px;
  font-weight: 700;
  color: var(--on-surface-strong);
}

.files-list-wrapper {
  flex-grow: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.files-list-wrapper h3 {
  font-size: 14px;
  color: var(--on-surface-strong);
  margin-bottom: var(--spacing-sm);
}

.no-changes {
  text-align: center;
  color: var(--muted);
  padding: var(--spacing-xl);
  flex-grow: 1;
}

.files-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.files-table th, .files-table td {
  padding: var(--spacing-sm);
  border-bottom: 1px solid var(--border);
}

.files-table th {
  background-color: var(--surface-variant);
  color: var(--on-surface-strong);
  font-weight: 600;
  text-align: left;
}

.file-path code {
  font-family: var(--font-mono);
  color: var(--on-surface-strong);
}

.text-right {
  text-align: right;
}
</style>
