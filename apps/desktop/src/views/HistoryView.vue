<template>
  <div class="history-container">
    <header class="view-header">
      <div>
        <h1>History & Trends</h1>
        <p class="subtitle" v-if="store.activeRepoDetail">
          Viewing purge trends and metrics for <strong>{{ store.activeRepoDetail.name }}</strong>
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
      <p>Please select a repository from the dropdown above to view trend history.</p>
    </div>

    <div v-else class="history-layout">
      <!-- Unsupported banner indicating P5 phase -->
      <div v-if="unsupportedMsg" class="unsupported-banner card">
        <div class="banner-header">
          <span class="info-icon">ℹ️</span>
          <h3>Phase 5 Integration Preview</h3>
        </div>
        <p class="banner-body">{{ unsupportedMsg }}</p>
        <p class="banner-hint">Below is a visual layout preview of the reporting dashboard coming in the next phase.</p>
      </div>

      <!-- Preview dashboard layout (Premium look, R7 parity) -->
      <div class="history-preview-dashboard">
        <div class="preview-grid">
          <div class="card preview-card">
            <h4>Total Branches Deleted</h4>
            <div class="val text-success">142</div>
            <p class="lbl">Across 12 cleanup sessions</p>
          </div>
          <div class="card preview-card">
            <h4>Disk Space Reclaimed</h4>
            <div class="val text-primary">1.24 GB</div>
            <p class="lbl">Through git-purge mirror compression</p>
          </div>
          <div class="card preview-card">
            <h4>Staleness Trend</h4>
            <div class="val text-warning">-34%</div>
            <p class="lbl">Stale branches reduced month-over-month</p>
          </div>
        </div>

        <div class="charts-section card">
          <h3>Stale Branches Cleanup Over Time</h3>
          <!-- Lightweight SVG Line Chart representing cleanup trend -->
          <div class="chart-wrapper">
            <svg viewBox="0 0 500 150" class="trend-chart">
              <!-- Grid lines -->
              <line x1="0" y1="30" x2="500" y2="30" stroke="var(--border)" stroke-dasharray="3,3" />
              <line x1="0" y1="75" x2="500" y2="75" stroke="var(--border)" stroke-dasharray="3,3" />
              <line x1="0" y1="120" x2="500" y2="120" stroke="var(--border)" stroke-dasharray="3,3" />

              <!-- Trend lines -->
              <path
                d="M 20 120 L 100 100 L 180 110 L 260 70 L 340 50 L 420 40 L 480 25"
                fill="none"
                stroke="var(--primary)"
                stroke-width="3"
                stroke-linecap="round"
                stroke-linejoin="round"
              />

              <!-- Highlight points -->
              <circle cx="20" cy="120" r="4" fill="var(--surface)" stroke="var(--primary)" stroke-width="2" />
              <circle cx="100" cy="100" r="4" fill="var(--surface)" stroke="var(--primary)" stroke-width="2" />
              <circle cx="180" cy="110" r="4" fill="var(--surface)" stroke="var(--primary)" stroke-width="2" />
              <circle cx="260" cy="70" r="4" fill="var(--surface)" stroke="var(--primary)" stroke-width="2" />
              <circle cx="340" cy="50" r="4" fill="var(--surface)" stroke="var(--primary)" stroke-width="2" />
              <circle cx="420" cy="40" r="4" fill="var(--surface)" stroke="var(--primary)" stroke-width="2" />
              <circle cx="480" cy="25" r="4" fill="var(--surface)" stroke="var(--primary)" stroke-width="2" />
            </svg>
          </div>
          <div class="chart-labels">
            <span>Jan</span>
            <span>Feb</span>
            <span>Mar</span>
            <span>Apr</span>
            <span>May</span>
            <span>Jun</span>
            <span>Jul</span>
          </div>
        </div>

        <div class="actions-panel card">
          <h3>Generate Audit Report</h3>
          <p class="description">Export a comprehensive log of branch cleanup trends and status metrics.</p>
          <div class="report-buttons">
            <button class="btn btn-secondary btn-sm" disabled>📥 Export Markdown</button>
            <button class="btn btn-secondary btn-sm" disabled>📥 Export JSON</button>
            <button class="btn btn-secondary btn-sm" disabled>📥 Export HTML</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import { useReposStore } from '../stores/repos';
import { historyGet } from '../api/ipc';

const store = useReposStore();
const selectedRepoId = ref(store.activeRepoId || '');
const unsupportedMsg = ref<string | null>(null);

const handleRepoChange = () => {
  if (selectedRepoId.value) {
    store.selectRepo(selectedRepoId.value);
    loadHistory();
  }
};

const loadHistory = async () => {
  if (!selectedRepoId.value) return;
  unsupportedMsg.value = null;
  try {
    await historyGet(selectedRepoId.value);
  } catch (err: any) {
    unsupportedMsg.value = err?.message || 'History command is currently unsupported.';
  }
};

watch(() => store.activeRepoId, (newId) => {
  if (newId) {
    selectedRepoId.value = newId;
    loadHistory();
  }
});

onMounted(() => {
  if (selectedRepoId.value) {
    loadHistory();
  }
});
</script>

<style scoped>
.history-container {
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

.history-layout {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
}

.unsupported-banner {
  background-color: rgba(97, 175, 239, 0.05);
  border: 1px solid rgba(97, 175, 239, 0.2);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.banner-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  color: var(--primary);
}

.banner-header h3 {
  font-size: 14px;
  font-weight: 600;
}

.banner-body {
  font-size: 13px;
  color: var(--on-surface);
}

.banner-hint {
  font-size: 11px;
  color: var(--muted);
}

.history-preview-dashboard {
  opacity: 0.65;
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
  pointer-events: none;
}

.preview-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: var(--spacing-md);
}

.preview-card h4 {
  font-size: 11px;
  text-transform: uppercase;
  color: var(--muted);
  letter-spacing: 0.5px;
}

.preview-card .val {
  font-size: 24px;
  font-weight: 700;
  margin: var(--spacing-xs) 0;
}

.preview-card .lbl {
  font-size: 11px;
  color: var(--muted);
}

.charts-section h3 {
  font-size: 14px;
  color: var(--on-surface-strong);
  margin-bottom: var(--spacing-sm);
}

.chart-wrapper {
  background-color: var(--surface-raised);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: var(--spacing-sm);
}

.trend-chart {
  width: 100%;
  height: 120px;
}

.chart-labels {
  display: flex;
  justify-content: space-between;
  padding: var(--spacing-xs) var(--spacing-md) 0;
  font-size: 11px;
  color: var(--muted);
}

.actions-panel h3 {
  font-size: 14px;
  color: var(--on-surface-strong);
  margin-bottom: 4px;
}

.actions-panel .description {
  font-size: 12px;
  color: var(--muted);
  margin-bottom: var(--spacing-sm);
}

.report-buttons {
  display: flex;
  gap: var(--spacing-sm);
}
</style>
