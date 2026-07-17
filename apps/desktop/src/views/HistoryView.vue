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
        <select id="repo-select" v-model="selectedRepoId" @change="handleRepoChange" :disabled="store.loading || loadingHistory || loadingRuns">
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
          <h3>Audit Reports</h3>
          <p class="description">Export a comprehensive log of branch cleanup trends and status metrics.</p>
          <div class="engine-buttons-wrapper">
            <button class="btn btn-primary w-100" @click="openReportModal" :disabled="store.loading || loadingHistory || loadingRuns">
              <ClipboardList class="lucide-icon" style="margin-right: 4px;" /> Generate Report
            </button>
          </div>
        </div>

        <!-- Executions Log Section -->
        <div class="executions-section card">
          <h3>Execution History & Logs</h3>
          <p class="description">Review past purge/scan runs and inspect the branches that were deleted or archived.</p>
          
          <div v-if="loadingRuns && runsData.length === 0" class="loading-state-inline">
            <span class="spinner spinner-sm"></span>
            <span>Loading past operations...</span>
          </div>
          
          <div v-else-if="runsData.length === 0" class="empty-runs">
            <p>No past execution logs found for this repository.</p>
          </div>
          
          <div v-else class="runs-table-wrapper">
            <table class="runs-table">
              <thead>
                <tr>
                  <th>Execution ID</th>
                  <th>Date & Time</th>
                  <th>Command</th>
                  <th>Mode</th>
                  <th>Deleted</th>
                  <th>Archived</th>
                  <th>Actor</th>
                  <th>Details</th>
                </tr>
              </thead>
              <tbody>
                <template v-for="run in runsData" :key="run.id">
                  <tr class="run-row" :class="{ expanded: expandedRuns.has(run.id) }">
                    <td class="code-font text-sm">{{ run.id.slice(0, 12) }}...</td>
                    <td>{{ formatDate(run.startedAt) }}</td>
                    <td>
                      <span class="badge" :class="getCommandBadgeClass(run.command)">
                        {{ run.command }}
                      </span>
                    </td>
                    <td>
                      <span class="badge" :class="run.mode === 'execute' ? 'badge-execute' : 'badge-dry'">
                        {{ run.mode }}
                      </span>
                    </td>
                    <td class="text-success font-bold">{{ run.deletedCount }}</td>
                    <td class="text-info font-bold">{{ run.archivedCount }}</td>
                    <td>{{ run.actor || 'system' }}</td>
                    <td>
                      <button 
                        v-if="run.branches && run.branches.length > 0" 
                        class="btn-icon" 
                        @click="toggleRunExpand(run.id)"
                        :disabled="store.loading || loadingHistory || loadingRuns"
                        style="display: inline-flex; align-items: center; gap: 4px;"
                      >
                        <ChevronUp v-if="expandedRuns.has(run.id)" class="lucide-icon" />
                        <ChevronDown v-else class="lucide-icon" />
                        {{ expandedRuns.has(run.id) ? 'Hide' : 'View Branches (' + run.branches.length + ')' }}
                      </button>
                      <span v-else class="text-muted text-sm">None</span>
                    </td>
                  </tr>
                  
                  <tr v-if="expandedRuns.has(run.id) && run.branches && run.branches.length > 0" class="details-row">
                    <td colspan="8">
                      <div class="expanded-details card">
                        <h4>Deleted/Archived Branches ({{ run.branches.length }})</h4>
                        <ul class="branches-list">
                          <li v-for="branch in run.branches" :key="branch" class="branch-item code-font">
                            <GitBranch class="lucide-icon" style="margin-right: 4px;" /> {{ branch }}
                          </li>
                        </ul>
                      </div>
                    </td>
                  </tr>
                </template>
              </tbody>
            </table>
            
            <div class="pagination-footer" v-if="hasMoreRuns || runsOffset > 0">
              <button 
                class="btn btn-secondary btn-sm" 
                :disabled="runsOffset === 0 || loadingRuns || store.loading || loadingHistory"
                @click="runsOffset -= runsLimit; loadRuns();"
                style="display: inline-flex; align-items: center; gap: 4px;"
              >
                <ChevronLeft class="lucide-icon" /> Previous
              </button>
              <span class="pagination-info">Page {{ Math.floor(runsOffset / runsLimit) + 1 }}</span>
              <button 
                class="btn btn-secondary btn-sm" 
                :disabled="!hasMoreRuns || loadingRuns || store.loading || loadingHistory"
                @click="loadNextPage()"
                style="display: inline-flex; align-items: center; gap: 4px;"
              >
                Next <ChevronRight class="lucide-icon" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Report Modal -->
    <div v-if="showReportModal" class="modal-overlay" @click.self="showReportModal = false">
      <div class="modal-card card report-modal-card">
        <header class="modal-header">
          <h3>Audit Reports</h3>
          <button class="close-btn" @click="showReportModal = false"><X class="lucide-icon" /></button>
        </header>

        <div class="report-tabs">
          <button 
            class="report-tab-btn" 
            :class="{ active: selectedReportType === 'audit' }" 
            @click="selectedReportType = 'audit'"
          >
            <ClipboardList class="lucide-icon" style="margin-right: 4px;" /> Branch Audit
          </button>
          <button 
            class="report-tab-btn" 
            :class="{ active: selectedReportType === 'trend' }" 
            @click="selectedReportType = 'trend'"
          >
            <TrendingUp class="lucide-icon" style="margin-right: 4px;" /> Cleanup Trend
          </button>
        </div>

        <main class="modal-body">
          <div v-if="generatingReport" class="loading-state">
            <span class="spinner"></span>
            <p>Generating markdown {{ selectedReportType }} report...</p>
          </div>
          <pre v-else class="report-preview"><code>{{ reportContent }}</code></pre>
        </main>
        <footer class="modal-footer">
          <button class="btn btn-secondary" @click="copyReportToClipboard"><ClipboardList class="lucide-icon" style="margin-right: 4px;" /> Copy to Clipboard</button>
          <button class="btn btn-primary" @click="downloadReportFile"><Download class="lucide-icon" style="margin-right: 4px;" /> Download File</button>
        </footer>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import { useReposStore } from '../stores/repos';
import { useHistoryStore } from '../stores/history';
import { ClipboardList, ChevronUp, ChevronDown, GitBranch, ChevronLeft, ChevronRight, X, TrendingUp, Download } from '@lucide/vue';
import { saveFile } from '../api/ipc';
import { save } from '@tauri-apps/plugin-dialog';
import { parseSafeDate, formatChartDate } from '../utils/date';

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

interface RunRecord {
  id: string;
  command: string;
  mode: string;
  startedAt: string;
  finishedAt: string | null;
  snapshotId: string | null;
  actor: string | null;
  deletedCount: number;
  archivedCount: number;
  branches: string[];
}

const store = useReposStore();
const historyStore = useHistoryStore();

const selectedRepoId = ref(store.activeRepoId || '');
const unsupportedMsg = ref<string | null>(null);

const historyData = computed(() => historyStore.historyData);
const loadingHistory = computed(() => historyStore.loading);
const runsData = computed(() => historyStore.runs);
const loadingRuns = computed(() => historyStore.loading);

// Past Runs / operations log state
const expandedRuns = ref<Set<string>>(new Set());
const runsLimit = 10;
const runsOffset = ref(0);
const hasMoreRuns = ref(true);

// Report Generation state
const showReportModal = ref(false);
const generatingReport = computed(() => historyStore.isGeneratingReport);
const reportContent = computed(() => historyStore.reportContent);
const selectedReportType = ref<'audit' | 'trend'>('audit');

const handleRepoChange = () => {
  if (selectedRepoId.value) {
    store.selectRepo(selectedRepoId.value);
    loadHistory();
  }
};

const toggleRunExpand = (runId: string) => {
  if (expandedRuns.value.has(runId)) {
    const next = new Set(expandedRuns.value);
    next.delete(runId);
    expandedRuns.value = next;
  } else {
    const next = new Set(expandedRuns.value);
    next.add(runId);
    expandedRuns.value = next;
  }
};

const getCommandBadgeClass = (command: string) => {
  if (command === 'delete') return 'badge-delete';
  if (command === 'archive') return 'badge-archive';
  return 'badge-scan';
};

const formatDate = (dateStr: string) => {
  try {
    const d = new Date(dateStr);
    return d.toLocaleString();
  } catch (e) {
    return dateStr;
  }
};

const loadRuns = async (reset = false) => {
  if (!selectedRepoId.value) return;
  if (reset) {
    runsOffset.value = 0;
  }
  expandedRuns.value = new Set();
  try {
    const raw = await historyStore.fetchRuns(selectedRepoId.value, runsLimit, runsOffset.value);
    const newRuns = raw as RunRecord[];
    hasMoreRuns.value = newRuns.length === runsLimit;
  } catch (err: any) {
    console.error('Failed to load past executions:', err);
  }
};

const loadNextPage = () => {
  if (loadingRuns.value || !hasMoreRuns.value) return;
  runsOffset.value += runsLimit;
  loadRuns();
};

const loadHistory = async () => {
  if (!selectedRepoId.value) return;
  unsupportedMsg.value = null;
  try {
    const raw = await historyStore.fetchHistory(selectedRepoId.value);
    // Sort chronologically
    historyStore.historyData = (raw as HistoryEntry[]).sort(
      (a, b) => parseSafeDate(a.recordedAt).getTime() - parseSafeDate(b.recordedAt).getTime()
    );
    await loadRuns(true);
  } catch (err: any) {
    unsupportedMsg.value = err?.message || 'History is currently unavailable.';
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
    const dateLabel = formatChartDate(entry.recordedAt);
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

// Report Generation Methods
const openReportModal = async () => {
  if (!selectedRepoId.value) return;
  showReportModal.value = true;
  await fetchReport();
};

const fetchReport = async () => {
  if (!selectedRepoId.value) return;
  try {
    await historyStore.generateReport(selectedRepoId.value, 'markdown', selectedReportType.value);
  } catch (err: any) {
    alert('Failed to generate report: ' + err.message);
    showReportModal.value = false;
  }
};

watch(selectedReportType, () => {
  if (showReportModal.value) {
    fetchReport();
  }
});

const copyReportToClipboard = async () => {
  try {
    await navigator.clipboard.writeText(reportContent.value);
    alert('Copied report content to clipboard!');
  } catch (err) {
    alert('Failed to copy: ' + err);
  }
};

const downloadReportFile = async () => {
  const activeRepoName = store.activeRepoDetail?.name || 'repo';
  const defaultFilename = `git-purge-${selectedReportType.value}-report-${activeRepoName}-${new Date().toISOString().split('T')[0]}.md`;

  try {
    const filePath = await save({
      filters: [
        {
          name: 'Markdown Report',
          extensions: ['md']
        }
      ],
      defaultPath: defaultFilename
    });

    if (filePath) {
      await saveFile(filePath, reportContent.value);
    }
  } catch (err: any) {
    console.error('Tauri save dialog failed, falling back to blob download:', err);
    try {
      const blob = new Blob([reportContent.value], { type: 'text/markdown' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = defaultFilename;
      a.click();
      URL.revokeObjectURL(url);
    } catch (fallbackErr: any) {
      alert('Failed to save report: ' + (fallbackErr?.message || fallbackErr));
    }
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

.history-live-dashboard {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-lg);
  margin-bottom: var(--spacing-lg);
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

.report-modal-card {
  width: 800px;
}

.report-tabs {
  display: flex;
  gap: var(--spacing-sm);
  border-bottom: 1px solid var(--border);
  margin-top: calc(-1 * var(--spacing-xs));
  margin-bottom: var(--spacing-sm);
}

.report-tab-btn {
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  color: var(--muted);
  font-size: 14px;
  font-weight: 500;
  padding: var(--spacing-sm) var(--spacing-md);
  cursor: pointer;
  transition: all 0.2s ease;
}

.report-tab-btn:hover {
  color: var(--on-surface-strong);
}

.report-tab-btn.active {
  color: var(--primary);
  border-bottom-color: var(--primary);
  font-weight: 600;
}

.executions-section h3 {
  font-size: 14px;
  color: var(--on-surface-strong);
  margin-bottom: 4px;
}

.executions-section .description {
  font-size: 12px;
  color: var(--muted);
  margin-bottom: var(--spacing-md);
}

.loading-state-inline {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  color: var(--muted);
  font-size: 13px;
  padding: var(--spacing-md) 0;
}

.spinner-sm {
  width: 16px;
  height: 16px;
  border-width: 2px;
}

.empty-runs {
  color: var(--muted);
  font-size: 13px;
  padding: var(--spacing-md) 0;
  text-align: center;
}

.runs-table-wrapper {
  overflow-x: auto;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
}

.runs-table {
  width: 100%;
  border-collapse: collapse;
  text-align: left;
  font-size: 13px;
}

.runs-table th {
  background-color: var(--surface-container);
  color: var(--muted);
  font-weight: 600;
  padding: var(--spacing-sm) var(--spacing-md);
  border-bottom: 1px solid var(--border);
}

.runs-table td {
  padding: var(--spacing-sm) var(--spacing-md);
  border-bottom: 1px solid var(--border);
  color: var(--on-surface);
}

.run-row:hover {
  background-color: rgba(255, 255, 255, 0.02);
}

.run-row.expanded {
  background-color: rgba(255, 255, 255, 0.01);
}

.code-font {
  font-family: var(--font-mono);
}

.font-bold {
  font-weight: 600;
}

.badge {
  display: inline-block;
  padding: 2px 6px;
  border-radius: var(--radius-xs);
  font-size: 11px;
  font-weight: 500;
  text-transform: capitalize;
}

.badge-delete {
  background-color: rgba(255, 180, 171, 0.1);
  color: var(--danger);
}

.badge-archive {
  background-color: rgba(167, 211, 135, 0.1);
  color: var(--info);
}

.badge-scan {
  background-color: rgba(255, 255, 255, 0.05);
  color: var(--muted);
}

.badge-execute {
  background-color: rgba(149, 204, 255, 0.1);
  color: var(--primary);
}

.badge-dry {
  background-color: rgba(255, 255, 255, 0.05);
  color: var(--muted);
}

.btn-icon {
  background: none;
  border: none;
  color: var(--primary);
  cursor: pointer;
  padding: 0;
  font-size: 13px;
  text-decoration: underline;
}

.btn-icon:hover {
  color: var(--on-surface-strong);
}

.details-row td {
  padding: 0 var(--spacing-md) var(--spacing-md);
  background-color: rgba(0, 0, 0, 0.15);
}

.expanded-details {
  background-color: var(--surface-container);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: var(--spacing-md);
  margin-top: var(--spacing-xs);
}

.expanded-details h4 {
  font-size: 12px;
  color: var(--on-surface-strong);
  margin-bottom: var(--spacing-sm);
}

.branches-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: var(--spacing-sm);
}

.branch-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  background-color: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-xs);
  padding: var(--spacing-xs) var(--spacing-sm);
  font-size: 12px;
  color: var(--on-surface);
}

.branch-icon {
  font-size: 14px;
}

.pagination-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-sm) var(--spacing-md);
  background-color: var(--surface-container);
  border-top: 1px solid var(--border);
}

.pagination-info {
  font-size: 12px;
  color: var(--muted);
}

.btn-sm {
  padding: 4px 8px;
  font-size: 12px;
}
</style>
