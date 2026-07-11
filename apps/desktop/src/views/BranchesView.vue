<template>
  <div class="branches-container">
    <header class="view-header">
      <div>
        <h1>Branches Explorer</h1>
        <p class="subtitle" v-if="store.activeRepoDetail">
          Managing <strong>{{ store.activeRepoDetail.name }}</strong> ({{ store.activeRepoDetail.localPath }})
        </p>
        <p class="subtitle" v-else>Select a repository to explore and analyze branches.</p>
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
      <p>Please select a repository from the dropdown above to begin branch analysis.</p>
    </div>

    <div v-else class="explorer-layout">
      <!-- Left sidebar: Scan & Filter controls -->
      <aside class="controls-panel card">
        <div class="control-group scan-box">
          <h3>Analysis Engine</h3>
          <!-- Scan Progress -->
          <div v-if="store.isScanning" class="scan-progress-container">
            <div class="progress-bar-bg">
              <div class="progress-bar-fill" :style="{ width: store.scanProgress + '%' }"></div>
            </div>
            <div class="progress-meta">
              <span class="progress-pct">{{ store.scanProgress }}%</span>
              <span class="progress-msg">{{ store.scanProgressMessage }}</span>
            </div>
            <button class="btn btn-secondary btn-sm cancel-btn" @click="store.cancelActiveTask">
              ✕ Cancel Scan
            </button>
          </div>

          <!-- Backup Progress -->
          <div v-else-if="isBackingUp" class="scan-progress-container">
            <div class="progress-bar-bg">
              <div class="progress-bar-fill" :style="{ width: backupProgress + '%' }"></div>
            </div>
            <div class="progress-meta">
              <span class="progress-pct">{{ backupProgress }}%</span>
              <span class="progress-msg">{{ backupProgressMessage }}</span>
            </div>
            <button class="btn btn-secondary btn-sm cancel-btn" @click="cancelBackup">
              ✕ Cancel Backup
            </button>
          </div>

          <!-- Normal State Buttons -->
          <div v-else class="engine-buttons-wrapper">
            <button class="btn btn-primary w-100 scan-main-btn" @click="triggerScan">
              🔄 Scan & Classify
            </button>
            <div class="engine-action-row">
              <button class="btn btn-secondary btn-sm flex-1" @click="openReportModal">
                📋 Generate Report
              </button>
              <button class="btn btn-secondary btn-sm flex-1" @click="triggerBackupSnapshot">
                💾 Create Snapshot
              </button>
            </div>
          </div>
          <p class="last-scanned" v-if="store.scannedAt">Last Scan: {{ formattedScannedAt }}</p>
        </div>

        <div class="control-group filter-box">
          <h3>Filters</h3>
          <div class="filter-item">
            <label for="search-input">Search Name</label>
            <input id="search-input" type="text" v-model="searchQuery" placeholder="e.g. feature/..." class="form-input" />
          </div>
          <div class="filter-item">
            <label for="filter-locality">Locality</label>
            <select id="filter-locality" v-model="filterLocality" class="form-input">
              <option value="all">All</option>
              <option value="local">Local Only</option>
              <option value="remote">Remote Only</option>
            </select>
          </div>
          <div class="filter-item">
            <label for="filter-freshness">Freshness</label>
            <select id="filter-freshness" v-model="filterFreshness" class="form-input">
              <option value="all">All</option>
              <option value="stale">Stale Only</option>
              <option value="active">Active Only</option>
            </select>
          </div>
          <div class="filter-item">
            <label for="filter-merge">Merge Status</label>
            <select id="filter-merge" v-model="filterMerge" class="form-input">
              <option value="all">All</option>
              <option value="merged">Merged Only</option>
              <option value="unmerged">Unmerged Only</option>
            </select>
          </div>
          <div class="filter-item">
            <label for="filter-protection">Protection</label>
            <select id="filter-protection" v-model="filterProtection" class="form-input">
              <option value="all">All</option>
              <option value="protected">Protected</option>
              <option value="unprotected">Unprotected</option>
            </select>
          </div>
          <div class="filter-item">
            <label for="filter-naming">Naming Policy</label>
            <select id="filter-naming" v-model="filterNaming" class="form-input">
              <option value="all">All</option>
              <option value="standard">Standard</option>
              <option value="nonStandard">Non-Standard</option>
            </select>
          </div>
        </div>

        <div class="control-group sort-box">
          <h3>Sorting</h3>
          <div class="filter-item">
            <label for="sort-select">Sort By</label>
            <select id="sort-select" v-model="sortBy" class="form-input">
              <option value="name">Branch Name</option>
              <option value="age">Age (Staleness)</option>
              <option value="commits">Committed Date</option>
              <option value="status">Status</option>
            </select>
          </div>
        </div>
      </aside>

      <!-- Main Branch List Area -->
      <section class="branches-list-area card">
        <div class="list-header">
          <h2>Detected Branches ({{ filteredBranches.length }})</h2>
          <div class="selection-actions" v-if="selectedBranches.length > 0">
            <span class="selection-count">{{ selectedBranches.length }} selected</span>
            <button class="btn btn-secondary btn-sm" @click="selectAllFiltered">Select All</button>
            <button class="btn btn-secondary btn-sm" @click="selectedBranches = []">Clear</button>
          </div>
        </div>

        <div class="table-wrapper">
          <table class="branches-table">
            <thead>
              <tr>
                <th width="40"><input type="checkbox" @change="toggleSelectAllFiltered" :checked="isAllFilteredSelected" /></th>
                <th>Branch / Ref</th>
                <th>Classification</th>
                <th>Age</th>
                <th>Committer</th>
              </tr>
            </thead>
            <tbody>
              <tr v-if="filteredBranches.length === 0">
                <td colspan="5" class="no-branches">
                  No branches found matching the current filters.
                </td>
              </tr>
              <tr v-for="branch in filteredBranches" :key="branch.name" :class="{ 'row-selected': selectedBranches.includes(branch.name) }">
                <td>
                  <div class="checkbox-wrapper">
                    <!-- SAFE-02: Protected refs cannot be selected for destructive actions -->
                    <input
                      type="checkbox"
                      :id="'chk-' + branch.name"
                      :value="branch.name"
                      v-model="selectedBranches"
                      :disabled="branch.classification.protected"
                    />
                    <span v-if="branch.classification.protected" class="lock-icon" title="Protected ref. Cannot delete or archive.">🔒</span>
                  </div>
                </td>
                <td>
                  <div class="branch-name-col">
                    <span class="branch-name" :class="{ 'protected-text': branch.classification.protected }">
                      {{ branch.name }}
                    </span>
                    <span class="tip-sha" v-if="branch.tipShort"><code>{{ branch.tipShort }}</code></span>
                  </div>
                  <div class="upstream-info" v-if="branch.upstream">
                    <span>tracks <code>{{ branch.upstream }}</code></span>
                    <span v-if="branch.classification.ahead > 0" class="badge badge-info badge-tiny">+{{ branch.classification.ahead }} ahead</span>
                    <span v-if="branch.classification.behind > 0" class="badge badge-warning badge-tiny">-{{ branch.classification.behind }} behind</span>
                  </div>
                </td>
                <td>
                  <div class="classification-badges">
                    <span :class="localityBadgeClass(branch)">{{ branch.classification.locality }}</span>
                    <span :class="freshnessBadgeClass(branch)">{{ branch.classification.freshness }}</span>
                    <span :class="mergeBadgeClass(branch)">{{ branch.classification.merge }}</span>
                    <span v-if="branch.classification.protected" class="badge badge-success">protected</span>
                    <span v-if="branch.classification.naming === 'nonStandard'" class="badge badge-purple">naming: non-std</span>
                  </div>
                </td>
                <td class="age-col">
                  {{ branch.ageDays }} days old
                </td>
                <td class="committer-col">
                  <div class="author">{{ branch.authorName }}</div>
                  <div class="date">{{ formattedDate(branch.committedAt) }}</div>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>
    </div>

    <!-- Action Drawer (Slide up when branches are selected) -->
    <div class="action-drawer" :class="{ 'drawer-open': selectedBranches.length > 0 }">
      <div class="drawer-content">
        <div class="drawer-left">
          <span class="drawer-title">Bulk Actions</span>
          <span class="drawer-meta">Selected: <strong>{{ selectedBranches.length }}</strong> branches</span>
        </div>
        <div class="drawer-right">
          <button class="btn btn-secondary" v-if="selectedBranches.length === 2" @click="triggerCompare">
            🔍 Compare Both
          </button>
          <button class="btn btn-secondary" @click="triggerBulkAction('archive')">
            📦 Archive Selected
          </button>
          <button class="btn btn-danger" @click="triggerBulkAction('delete')">
            🗑️ Purge/Delete Selected
          </button>
        </div>
      </div>
    </div>

    <!-- Report Modal -->
    <div v-if="showReportModal" class="modal-overlay" @click.self="showReportModal = false">
      <div class="modal-card card report-modal-card">
        <header class="modal-header">
          <h3>Audit Report (MARKDOWN)</h3>
          <button class="close-btn" @click="showReportModal = false">✕</button>
        </header>
        <main class="modal-body">
          <div v-if="generatingReport" class="loading-state">
            <span class="spinner"></span>
            <p>Generating standardized audit report...</p>
          </div>
          <pre v-else class="report-preview"><code>{{ reportContent }}</code></pre>
        </main>
        <footer class="modal-footer">
          <button class="btn btn-secondary" @click="copyReportToClipboard">📋 Copy to Clipboard</button>
          <button class="btn btn-primary" @click="downloadReportFile">📥 Download File</button>
        </footer>
      </div>
    </div>

    <!-- Duplicate Warning Modal -->
    <div v-if="showDuplicateWarning" class="modal-overlay" @click.self="showDuplicateWarning = false">
      <div class="modal-card card warning-modal-card">
        <header class="modal-header">
          <h3>⚠️ Duplicate Snapshot Warning</h3>
          <button class="close-btn" @click="showDuplicateWarning = false">✕</button>
        </header>
        <main class="modal-body warning-body">
          <p>No branches or commit tips have changed since the last backup snapshot.</p>
          <div class="duplicate-details">
            <p><strong>Latest Snapshot:</strong> <code>{{ latestSnapshotId }}</code></p>
            <p><strong>Created At:</strong> {{ formattedLatestSnapshotDate }}</p>
          </div>
          <p class="warning-alert-text">Creating a new snapshot now will duplicate identical reference files on disk. Do you want to proceed anyway?</p>
        </main>
        <footer class="modal-footer">
          <button class="btn btn-secondary" @click="showDuplicateWarning = false">Cancel</button>
          <button class="btn btn-danger" @click="proceedWithBackup">Yes, Proceed</button>
        </footer>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useReposStore } from '../stores/repos';
import { 
  type Branch,
  reportGenerate, 
  backupCreate, 
  backupList, 
  backupShow, 
  listenProgress, 
  cancel 
} from '../api/ipc';

const router = useRouter();
const store = useReposStore();

const selectedRepoId = ref(store.activeRepoId || '');
const selectedBranches = ref<string[]>([]);

// Backup Snapshot state
const isBackingUp = ref(false);
const backupProgress = ref(0);
const backupProgressMessage = ref('');
const activeBackupTaskId = ref('');

// Duplicate warning details
const showDuplicateWarning = ref(false);
const latestSnapshotId = ref('');
const latestSnapshotDate = ref('');

const formattedLatestSnapshotDate = computed(() => {
  if (!latestSnapshotDate.value) return '';
  const d = new Date(latestSnapshotDate.value);
  if (isNaN(d.getTime())) return latestSnapshotDate.value;
  return d.toLocaleString();
});

// Report Generation state
const showReportModal = ref(false);
const generatingReport = ref(false);
const reportContent = ref('');

// Filter and Sort inputs
const searchQuery = ref('');
const filterLocality = ref('all');
const filterFreshness = ref('all');
const filterMerge = ref('all');
const filterProtection = ref('all');
const filterNaming = ref('all');
const sortBy = ref('name');

const formattedScannedAt = computed(() => {
  if (!store.scannedAt) return '';
  try {
    return new Date(store.scannedAt).toLocaleTimeString();
  } catch {
    return store.scannedAt;
  }
});

const formattedDate = (dateStr: string) => {
  if (!dateStr) return '';
  const d = new Date(dateStr);
  if (isNaN(d.getTime())) {
    return dateStr;
  }
  return d.toLocaleDateString();
};

const triggerScan = async () => {
  if (store.activeRepoId) {
    selectedBranches.value = [];
    try {
      await store.runScan(store.activeRepoId, { includeRemote: true });
    } catch (err: any) {
      alert('Scan failed: ' + err.message);
    }
  }
};

// Report Generation Methods
const openReportModal = async () => {
  if (!store.activeRepoId) return;
  showReportModal.value = true;
  generatingReport.value = true;
  reportContent.value = '';
  
  try {
    const res = await reportGenerate(store.activeRepoId, 'markdown');
    reportContent.value = res.content;
  } catch (err: any) {
    alert('Failed to generate report: ' + err.message);
    showReportModal.value = false;
  } finally {
    generatingReport.value = false;
  }
};

const copyReportToClipboard = async () => {
  try {
    await navigator.clipboard.writeText(reportContent.value);
    alert('Copied report content to clipboard!');
  } catch (err) {
    alert('Failed to copy: ' + err);
  }
};

const downloadReportFile = () => {
  const activeRepoName = store.activeRepoDetail?.name || 'repo';
  const blob = new Blob([reportContent.value], { type: 'text/markdown' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `git-purge-report-${activeRepoName}-${new Date().toISOString().split('T')[0]}.md`;
  a.click();
  URL.revokeObjectURL(url);
};

// Snapshot Backup Methods
const triggerBackupSnapshot = async () => {
  if (!store.activeRepoId) return;
  
  try {
    const list = await backupList(store.activeRepoId);
    if (list && list.length > 0) {
      // Sort desc so latest is first
      const sorted = [...list].sort((a, b) => b.id.localeCompare(a.id));
      const latest = sorted[0];
      
      const details = await backupShow(latest.id);
      if (details && details.refs) {
        const currentBranches = store.branches;
        let isDuplicate = true;
        
        if (currentBranches.length !== details.refs.length) {
          isDuplicate = false;
        } else {
          for (const branch of currentBranches) {
            const matchedRef = details.refs.find(r => r.branch === branch.name);
            if (!matchedRef || matchedRef.tipSha !== branch.tipSha) {
              isDuplicate = false;
              break;
            }
          }
        }
        
        if (isDuplicate) {
          latestSnapshotId.value = latest.id;
          latestSnapshotDate.value = latest.createdAt;
          showDuplicateWarning.value = true;
          return;
        }
      }
    }
    
    await proceedWithBackup();
  } catch (err: any) {
    alert('Backup check failed: ' + err.message);
  }
};

const proceedWithBackup = async () => {
  showDuplicateWarning.value = false;
  if (!store.activeRepoId) return;

  isBackingUp.value = true;
  backupProgress.value = 0;
  backupProgressMessage.value = 'Preparing snapshot...';
  
  const taskId = 'backup-' + Math.random().toString(36).slice(2, 7);
  activeBackupTaskId.value = taskId;
  
  let unsubscribe: (() => void) | null = null;
  
  try {
    unsubscribe = await listenProgress((evt: any) => {
      if (evt.taskId === taskId) {
        backupProgress.value = evt.pct;
        backupProgressMessage.value = evt.message;
      }
    });

    const options = {
      trigger: 'manual' as const,
      verify: true,
      refs: []
    };
    
    const snapshot = await backupCreate(store.activeRepoId, options, taskId);
    alert(`Snapshot backup created successfully!\nID: ${snapshot.id}\nRefs: ${snapshot.refCount}`);
    
    await store.selectRepo(store.activeRepoId);
  } catch (err: any) {
    if (err?.message !== 'CANCELLED') {
      alert('Snapshot backup failed: ' + (err?.message || err));
    }
  } finally {
    isBackingUp.value = false;
    activeBackupTaskId.value = '';
    if (unsubscribe) {
      unsubscribe();
    }
  }
};

const cancelBackup = async () => {
  if (activeBackupTaskId.value) {
    try {
      await cancel(activeBackupTaskId.value);
    } catch (err: any) {
      console.error('Failed to cancel backup:', err);
    }
  }
};

const handleRepoChange = () => {
  if (selectedRepoId.value) {
    selectedBranches.value = [];
    store.selectRepo(selectedRepoId.value);
  }
};

// Badges Classes
const localityBadgeClass = (b: Branch) => {
  return b.classification.locality === 'local' ? 'badge badge-info' : 'badge badge-purple';
};

const freshnessBadgeClass = (b: Branch) => {
  return b.classification.freshness === 'stale' ? 'badge badge-warning' : 'badge badge-success';
};

const mergeBadgeClass = (b: Branch) => {
  return b.classification.merge === 'merged' ? 'badge badge-success' : 'badge badge-danger';
};

// Filtered and Sorted Branches
const filteredBranches = computed(() => {
  let list = [...store.branches];

  // 1. Text Query
  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase().trim();
    list = list.filter(b => b.name.toLowerCase().includes(q));
  }

  // 2. Locality Filter
  if (filterLocality.value !== 'all') {
    list = list.filter(b => b.classification.locality === filterLocality.value);
  }

  // 3. Freshness Filter
  if (filterFreshness.value !== 'all') {
    list = list.filter(b => b.classification.freshness === filterFreshness.value);
  }

  // 4. Merge Status Filter
  if (filterMerge.value !== 'all') {
    list = list.filter(b => b.classification.merge === filterMerge.value);
  }

  // 5. Protection Filter
  if (filterProtection.value !== 'all') {
    const isProt = filterProtection.value === 'protected';
    list = list.filter(b => b.classification.protected === isProt);
  }

  // 6. Naming Policy Filter
  if (filterNaming.value !== 'all') {
    list = list.filter(b => b.classification.naming === filterNaming.value);
  }

  // 7. Sort
  list.sort((a, b) => {
    if (sortBy.value === 'name') {
      return a.name.localeCompare(b.name);
    } else if (sortBy.value === 'age') {
      return b.ageDays - a.ageDays;
    } else if (sortBy.value === 'commits') {
      return new Date(b.committedAt).getTime() - new Date(a.committedAt).getTime();
    } else if (sortBy.value === 'status') {
      const aProt = a.classification.protected ? 1 : 0;
      const bProt = b.classification.protected ? 1 : 0;
      if (aProt !== bProt) return bProt - aProt;
      const aStale = a.classification.freshness === 'stale' ? 1 : 0;
      const bStale = b.classification.freshness === 'stale' ? 1 : 0;
      return bStale - aStale;
    }
    return 0;
  });

  return list;
});

// Selection Helpers
const selectableFilteredBranches = computed(() => {
  return filteredBranches.value.filter(b => !b.classification.protected);
});

const isAllFilteredSelected = computed(() => {
  const selectable = selectableFilteredBranches.value;
  if (selectable.length === 0) return false;
  return selectable.every(b => selectedBranches.value.includes(b.name));
});

const toggleSelectAllFiltered = () => {
  const selectable = selectableFilteredBranches.value;
  if (isAllFilteredSelected.value) {
    selectedBranches.value = selectedBranches.value.filter(
      name => !selectable.some(b => b.name === name)
    );
  } else {
    selectable.forEach(b => {
      if (!selectedBranches.value.includes(b.name)) {
        selectedBranches.value.push(b.name);
      }
    });
  }
};

const selectAllFiltered = () => {
  selectableFilteredBranches.value.forEach(b => {
    if (!selectedBranches.value.includes(b.name)) {
      selectedBranches.value.push(b.name);
    }
  });
};

const triggerCompare = () => {
  if (selectedBranches.value.length === 2 && store.activeRepoId) {
    router.push({
      path: '/diff',
      query: {
        repoId: store.activeRepoId,
        branchA: selectedBranches.value[0],
        branchB: selectedBranches.value[1]
      }
    });
  }
};

const triggerBulkAction = (action: 'delete' | 'archive') => {
  if (selectedBranches.value.length > 0 && store.activeRepoId) {
    router.push({
      path: '/plan',
      query: {
        repoId: store.activeRepoId,
        actionKind: action,
        refs: selectedBranches.value.join(',')
      }
    });
  }
};

watch(() => store.activeRepoId, (newId) => {
  if (newId) {
    selectedRepoId.value = newId;
  }
});
</script>

<style scoped>
.branches-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  padding: var(--spacing-lg);
  gap: var(--spacing-md);
  position: relative;
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

.explorer-layout {
  display: flex;
  gap: var(--spacing-md);
  flex-grow: 1;
  overflow: hidden;
}

/* Controls Panel */
.controls-panel {
  width: 280px;
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
  flex-shrink: 0;
  overflow-y: auto;
}

.control-group h3 {
  font-size: 11px;
  color: var(--muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: var(--spacing-sm);
  border-bottom: 1px solid var(--border);
  padding-bottom: 4px;
}

.w-100 {
  width: 100%;
}

.scan-box {
  display: flex;
  flex-direction: column;
}

.scan-progress-container {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.progress-bar-bg {
  width: 100%;
  height: 6px;
  background-color: var(--border);
  border-radius: var(--radius-round);
  overflow: hidden;
}

.progress-bar-fill {
  height: 100%;
  background-color: var(--primary);
  transition: width 0.2s ease;
}

.progress-meta {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
  color: var(--on-surface);
}

.progress-msg {
  color: var(--muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 180px;
}

.cancel-btn {
  margin-top: var(--spacing-xs);
}

.last-scanned {
  font-size: 10px;
  color: var(--muted);
  margin-top: 4px;
  text-align: right;
}

.filter-box, .sort-box {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.filter-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.filter-item label {
  font-size: 11px;
  color: var(--on-surface);
}

.form-input {
  background-color: var(--surface-raised);
  color: var(--on-surface);
  border: 1px solid var(--border);
  padding: 6px var(--spacing-xs);
  border-radius: var(--radius-xs);
  outline: none;
  font-size: 12px;
}

.form-input:focus {
  border-color: var(--primary);
}

/* Branch List Area */
.branches-list-area {
  flex-grow: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-sm);
  flex-shrink: 0;
}

.list-header h2 {
  font-size: 16px;
  color: var(--on-surface-strong);
}

.selection-actions {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
}

.selection-count {
  font-size: 12px;
  color: var(--muted);
  margin-right: var(--spacing-xs);
}

.table-wrapper {
  flex-grow: 1;
  overflow-y: auto;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
}

.branches-table {
  width: 100%;
  border-collapse: collapse;
  text-align: left;
  font-size: 13px;
}

.branches-table th, .branches-table td {
  padding: var(--spacing-sm);
  border-bottom: 1px solid var(--border);
}

.branches-table th {
  background-color: var(--surface-variant);
  color: var(--on-surface-strong);
  font-weight: 600;
  position: sticky;
  top: 0;
  z-index: 1;
}

.no-branches {
  text-align: center;
  color: var(--muted);
  padding: var(--spacing-xl);
}

.row-selected {
  background-color: rgba(97, 175, 239, 0.05);
}

.checkbox-wrapper {
  display: flex;
  align-items: center;
  gap: 4px;
}

.lock-icon {
  font-size: 12px;
  cursor: help;
}

.branch-name-col {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.branch-name {
  font-family: var(--font-mono);
  font-weight: 500;
  color: var(--on-surface-strong);
}

.protected-text {
  color: var(--success);
}

.tip-sha code {
  font-size: 11px;
  background-color: var(--surface-raised);
  padding: 1px 4px;
  border-radius: var(--radius-xs);
  color: var(--muted);
}

.upstream-info {
  font-size: 11px;
  color: var(--muted);
  margin-top: 2px;
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
}

.badge-tiny {
  padding: 0px 4px;
  font-size: 9px;
  line-height: 12px;
}

.classification-badges {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.age-col {
  color: var(--on-surface);
}

.committer-col .author {
  font-weight: 500;
}

.committer-col .date {
  font-size: 11px;
  color: var(--muted);
}

/* Action Drawer */
.action-drawer {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  background-color: var(--surface-raised);
  border-top: 2px solid var(--primary);
  box-shadow: 0 -4px 12px rgba(0, 0, 0, 0.3);
  transform: translateY(100%);
  transition: transform var(--transition-normal);
  z-index: 20;
}

.drawer-open {
  transform: translateY(0);
}

.drawer-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-md) var(--spacing-lg);
  max-width: 1200px;
  margin: 0 auto;
}

.drawer-left {
  display: flex;
  flex-direction: column;
}

.drawer-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--on-surface-strong);
}

.drawer-meta {
  font-size: 12px;
  color: var(--muted);
}

.drawer-right {
  display: flex;
  gap: var(--spacing-sm);
}

/* Engine Action Row styling */
.engine-buttons-wrapper {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.engine-action-row {
  display: flex;
  gap: var(--spacing-xs);
}

.scan-main-btn {
  margin-bottom: 2px;
}

.flex-1 {
  flex: 1;
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
  width: 700px;
  max-width: 100%;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
  border: 1px solid var(--border);
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.5);
  background-color: var(--surface);
  border-radius: var(--radius-sm);
  padding: var(--spacing-lg);
}

.report-modal-card {
  width: 800px;
}

.warning-modal-card {
  width: 500px;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid var(--border);
  padding-bottom: var(--spacing-sm);
}

.modal-header h3 {
  font-size: 16px;
  color: var(--on-surface-strong);
  margin: 0;
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
  max-height: 50vh;
  overflow-y: auto;
  margin: 0;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--spacing-sm);
  border-top: 1px solid var(--border);
  padding-top: var(--spacing-md);
}

/* Duplicate warning body */
.warning-body {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
  font-size: 13px;
  color: var(--on-surface);
}

.duplicate-details {
  background-color: var(--surface-raised);
  border: 1px solid var(--border);
  padding: var(--spacing-sm);
  border-radius: var(--radius-xs);
  font-family: var(--font-mono);
  font-size: 12px;
}

.duplicate-details p {
  margin: var(--spacing-xs) 0;
}

.warning-alert-text {
  color: var(--danger);
  font-weight: 500;
}
</style>
