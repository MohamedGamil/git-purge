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
      <!-- Loading state -->
      <div v-if="loadingHistory" class="loading-state card">
        <span class="spinner"></span>
        <p>Loading history trends...</p>
      </div>

      <!-- Error state -->
      <div v-else-if="unsupportedMsg" class="error-state card">
        <h3>History Fetch Failed</h3>
        <p class="error-msg">{{ unsupportedMsg }}</p>
      </div>

      <!-- No history data available -->
      <div v-else-if="!historyData || historyData.length === 0" class="empty-state card">
        <p>No historical trends recorded for this repository yet. Run cleanup operations to seed history.</p>
      </div>

      <!-- Live Dashboard -->
      <div v-else class="history-live-dashboard">
        <div class="stats-grid">
          <div class="card stat-card">
            <h4>Total Branches Deleted</h4>
            <div class="val text-success">{{ totalDeleted }}</div>
            <p class="lbl">Across all recorded sessions</p>
          </div>
          <div class="card stat-card">
            <h4>Total Branches Archived</h4>
            <div class="val text-info">{{ totalArchived }}</div>
            <p class="lbl">Safe reference cold storage</p>
          </div>
          <div class="card stat-card">
            <h4>Current Stale Branches</h4>
            <div class="val text-warning">{{ currentStaleCount }}</div>
            <p class="lbl">Latest recorded metrics</p>
          </div>
        </div>

        <div class="charts-section card">
          <h3>Stale Branches Cleanup Over Time</h3>
          
          <div class="chart-wrapper">
            <svg viewBox="0 0 500 150" class="trend-chart">
              <!-- Grid lines -->
              <line x1="0" y1="30" x2="500" y2="30" stroke="var(--border)" stroke-dasharray="3,3" />
              <line x1="0" y1="75" x2="500" y2="75" stroke="var(--border)" stroke-dasharray="3,3" />
              <line x1="0" y1="120" x2="500" y2="120" stroke="var(--border)" stroke-dasharray="3,3" />

              <!-- Trend line path -->
              <path
                v-if="chartPathD"
                :d="chartPathD"
                fill="none"
                stroke="var(--primary)"
                stroke-width="3"
                stroke-linecap="round"
                stroke-linejoin="round"
              />

              <!-- Highlight points -->
              <circle
                v-for="(pt, idx) in chartPoints"
                :key="'pt-' + idx"
                :cx="pt.x"
                :cy="pt.y"
                r="4"
                fill="var(--surface)"
                stroke="var(--primary)"
                stroke-width="2"
              />

              <!-- Value labels on top of points -->
              <text
                v-for="(pt, idx) in chartPoints"
                :key="'lbl-' + idx"
                :x="pt.x"
                :y="pt.y - 8"
                font-size="9"
                fill="var(--on-surface)"
                text-anchor="middle"
              >
                {{ pt.value }}
              </text>
            </svg>
          </div>
          
          <div class="chart-labels">
            <span v-for="(pt, idx) in chartPoints" :key="'lbl-date-' + idx">
              {{ pt.label }}
            </span>
          </div>
        </div>

        <div class="actions-panel card">
          <h3>Generate Audit Report</h3>
          <p class="description">Export a comprehensive log of branch cleanup trends and status metrics.</p>
          <div class="report-buttons">
            <button class="btn btn-secondary btn-sm" @click="exportReport('markdown')" :disabled="exporting">
              📥 Export Markdown
            </button>
            <button class="btn btn-secondary btn-sm" @click="exportReport('json')" :disabled="exporting">
              📥 Export JSON
            </button>
            <button class="btn btn-secondary btn-sm" @click="exportReport('html')" :disabled="exporting">
              📥 Export HTML
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Report Modal -->
    <div v-if="showReportModal" class="modal-overlay" @click.self="showReportModal = false">
      <div class="modal-card card">
        <header class="modal-header">
          <h3>Audit Report ({{ reportFormat.toUpperCase() }})</h3>
          <button class="close-btn" @click="showReportModal = false">✕</button>
        </header>
        <main class="modal-body">
          <pre class="report-preview"><code>{{ reportContent }}</code></pre>
        </main>
        <footer class="modal-footer">
          <button class="btn btn-secondary" @click="copyToClipboard">📋 Copy to Clipboard</button>
          <button class="btn btn-primary" @click="downloadReportFile">📥 Download File</button>
        </footer>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import { useReposStore } from '../stores/repos';
import { historyGet, reportGenerate } from '../api/ipc';

interface HistoryEntry {
  recordedAt: string;
  totalBranches: number;
  activeCount: number;
  staleCount: number;
  mergedCount: number;
  unmergedCount: number;
  deletedCount: number;
  archivedCount: number;
  nonStandardCount: number;
}

const store = useReposStore();
const selectedRepoId = ref(store.activeRepoId || '');
const loadingHistory = ref(false);
const unsupportedMsg = ref<string | null>(null);
const historyData = ref<HistoryEntry[]>([]);

// Export report modal state
const exporting = ref(false);
const showReportModal = ref(false);
const reportContent = ref('');
const reportFormat = ref('');

const handleRepoChange = () => {
  if (selectedRepoId.value) {
    store.selectRepo(selectedRepoId.value);
    loadHistory();
  }
};

const loadHistory = async () => {
  if (!selectedRepoId.value) return;
  loadingHistory.value = true;
  unsupportedMsg.value = null;
  historyData.value = [];
  try {
    const raw = await historyGet(selectedRepoId.value);
    // Sort chronologically
    historyData.value = (raw as HistoryEntry[]).sort(
      (a, b) => new Date(a.recordedAt).getTime() - new Date(b.recordedAt).getTime()
    );
  } catch (err: any) {
    unsupportedMsg.value = err?.message || 'History is currently unavailable.';
  } finally {
    loadingHistory.value = false;
  }
};

// Calculations
const totalDeleted = computed(() => {
  return historyData.value.reduce((acc, h) => acc + (h.deletedCount || 0), 0);
});

const totalArchived = computed(() => {
  return historyData.value.reduce((acc, h) => acc + (h.archivedCount || 0), 0);
});

const currentStaleCount = computed(() => {
  if (historyData.value.length === 0) return 0;
  return historyData.value[historyData.value.length - 1].staleCount;
});

// SVG Chart Calculations
const chartPoints = computed(() => {
  if (historyData.value.length === 0) return [];
  const entries = historyData.value;
  const maxVal = Math.max(...entries.map(e => e.staleCount), 1);
  const width = 500;
  const height = 120;

  return entries.map((entry, index) => {
    const x = entries.length > 1
      ? (index / (entries.length - 1)) * (width - 40) + 20
      : width / 2;
    const y = height - (entry.staleCount / maxVal) * (height - 40) - 20;
    const dateLabel = new Date(entry.recordedAt).toLocaleDateString(undefined, {
      month: 'short',
      day: 'numeric'
    });
    return { x, y, value: entry.staleCount, label: dateLabel };
  });
});

const chartPathD = computed(() => {
  const pts = chartPoints.value;
  if (pts.length === 0) return '';
  if (pts.length === 1) {
    return `M ${pts[0].x - 5} ${pts[0].y} L ${pts[0].x + 5} ${pts[0].y}`;
  }
  return pts.reduce((acc, pt, idx) => {
    return acc + (idx === 0 ? `M ${pt.x} ${pt.y}` : ` L ${pt.x} ${pt.y}`);
  }, '');
});

// Export functions
const exportReport = async (format: string) => {
  if (!selectedRepoId.value) return;
  exporting.value = true;
  try {
    const res = await reportGenerate(selectedRepoId.value, format);
    reportContent.value = res.content;
    reportFormat.value = format;
    showReportModal.value = true;
  } catch (err: any) {
    alert('Report generation failed: ' + (err?.message || err));
  } finally {
    exporting.value = false;
  }
};

const copyToClipboard = async () => {
  try {
    await navigator.clipboard.writeText(reportContent.value);
    alert('Copied report content to clipboard!');
  } catch (err) {
    alert('Failed to copy: ' + err);
  }
};

const downloadReportFile = () => {
  const activeRepoName = store.activeRepoDetail?.name || 'repo';
  const ext = reportFormat.value === 'json' ? 'json' : reportFormat.value === 'html' ? 'html' : 'md';
  const mime = reportFormat.value === 'json' ? 'application/json' : reportFormat.value === 'html' ? 'text/html' : 'text/markdown';
  const blob = new Blob([reportContent.value], { type: mime });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `git-purge-report-${activeRepoName}-${new Date().toISOString().split('T')[0]}.${ext}`;
  a.click();
  URL.revokeObjectURL(url);
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

.loading-state, .error-state, .empty-state {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  padding: var(--spacing-xl);
  color: var(--muted);
  gap: var(--spacing-md);
}

.spinner {
  width: 24px;
  height: 24px;
  border: 3px solid var(--border);
  border-top-color: var(--primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.error-msg {
  color: var(--danger);
  font-family: var(--font-mono);
  font-size: 13px;
}

/* Stats grid */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: var(--spacing-md);
}

.stat-card h4 {
  font-size: 11px;
  text-transform: uppercase;
  color: var(--muted);
  letter-spacing: 0.5px;
}

.stat-card .val {
  font-size: 28px;
  font-weight: 700;
  margin: var(--spacing-xs) 0;
}

.stat-card .lbl {
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
  padding: var(--spacing-md);
}

.trend-chart {
  width: 100%;
  height: 120px;
  overflow: visible;
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

/* Modal styling */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.6);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
  padding: var(--spacing-lg);
}

.modal-card {
  width: 600px;
  max-width: 100%;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
  border: 1px solid var(--border);
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.3);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid var(--border);
  padding-bottom: var(--spacing-xs);
}

.modal-header h3 {
  font-size: 16px;
  color: var(--on-surface-strong);
}

.close-btn {
  background: none;
  border: none;
  font-size: 18px;
  color: var(--muted);
  cursor: pointer;
  padding: 4px;
}

.close-btn:hover {
  color: var(--on-surface-strong);
}

.modal-body {
  overflow-y: auto;
  flex-grow: 1;
}

.report-preview {
  background-color: var(--surface-variant);
  border: 1px solid var(--border);
  padding: var(--spacing-md);
  border-radius: var(--radius-xs);
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--on-surface);
  white-space: pre-wrap;
  word-break: break-all;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--spacing-sm);
  border-top: 1px solid var(--border);
  padding-top: var(--spacing-md);
}
</style>
