<template>
  <div class="backups-container">
    <header class="view-header">
      <div>
        <h1>Backups & Restore Points</h1>
        <p class="subtitle" v-if="store.activeRepoDetail">
          Managing backups for <strong>{{ store.activeRepoDetail.name }}</strong>
        </p>
        <p class="subtitle" v-else>Select a repository to browse backups.</p>
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
      <p>Please select a repository from the dropdown above to view its backup snapshots.</p>
    </div>

    <div v-else class="backups-layout">
      <!-- Snapshots list -->
      <section class="snapshots-list card">
        <div class="list-header">
          <h2>Backup Snapshots ({{ snapshots.length }})</h2>
          <button class="btn btn-secondary btn-sm" @click="fetchSnapshots" :disabled="loading">
            🔄 Refresh
          </button>
        </div>

        <div v-if="loading && !snapshots.length" class="loading-state">
          <span class="spinner"></span> Loading snapshots...
        </div>

        <div v-else-if="snapshots.length === 0" class="empty-state">
          <p>No snapshots found for this repository. Pre-deletion backups are created automatically before any destructive action unless disabled.</p>
        </div>

        <div v-else class="snapshots-grid">
          <div
            v-for="snap in snapshots"
            :key="snap.id"
            class="snap-card card"
            :class="{ 'snap-active': activeSnapshotId === snap.id }"
          >
            <div class="snap-header" @click="toggleSnapshot(snap.id)">
              <div class="snap-meta">
                <span class="snap-id"><code>{{ snap.id }}</code></span>
                <span class="badge badge-tiny" :class="triggerBadgeClass(snap.trigger)">
                  {{ snap.trigger }}
                </span>
                <span class="snap-date">{{ formattedDate(snap.createdAt) }}</span>
              </div>
              <div class="snap-right">
                <span class="ref-count">{{ snap.refCount }} refs</span>
                <span class="arrow" :class="{ 'arrow-down': activeSnapshotId === snap.id }">▶</span>
              </div>
            </div>

            <!-- Expanded Details -->
            <div v-if="activeSnapshotId === snap.id" class="snap-details">
              <div class="snap-actions">
                <button
                  class="btn btn-secondary btn-sm"
                  @click="verifySnapshot(snap.id)"
                  :disabled="verifyingId === snap.id"
                >
                  <span v-if="verifyingId === snap.id">Verifying...</span>
                  <span v-else>🔍 Verify Integrity</span>
                </button>
              </div>

              <!-- Verification Results -->
              <div v-if="verifyResults && verifyResults.snapshotId === snap.id" class="verify-results-box" :class="verifyResults.ok ? 'verify-ok' : 'verify-failed'">
                <p v-if="verifyResults.ok">✅ Snapshot integrity verified. {{ verifyResults.checkedRefs }} refs checked successfully.</p>
                <div v-else>
                  <p class="text-danger">❌ Integrity problems detected ({{ verifyResults.problems.length }}):</p>
                  <ul>
                    <li v-for="prob in verifyResults.problems" :key="prob">{{ prob }}</li>
                  </ul>
                </div>
              </div>

              <!-- Snapshot Refs List -->
              <div v-if="loadingDetails" class="loading-state-sm">
                <span class="spinner spinner-sm"></span> Loading references...
              </div>
              <div v-else-if="snapshotDetail" class="refs-table-wrapper">
                <table class="refs-table">
                  <thead>
                    <tr>
                      <th>Ref / Branch</th>
                      <th>Tip Commit SHA</th>
                      <th>Commits</th>
                      <th>Merge State</th>
                      <th>Action</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr v-for="ref in snapshotDetail.refs" :key="ref.originalRef || ref.branch">
                      <td>
                        <span class="branch-name"><code>{{ ref.branch }}</code></span>
                        <span :class="ref.locality === 'local' ? 'badge badge-info badge-tiny' : 'badge badge-purple badge-tiny'" style="margin-left: var(--spacing-xs);">
                          {{ ref.locality }}
                        </span>
                        <span class="upstream-lbl" v-if="ref.upstream">tracks <code>{{ ref.upstream }}</code></span>
                      </td>
                      <td><code>{{ ref.tipSha.substring(0, 7) }}</code></td>
                      <td>{{ ref.commitCount }} commits</td>
                      <td>
                        <span class="badge badge-tiny" :class="ref.merge === 'merged' ? 'badge-success' : 'badge-danger'">
                          {{ ref.merge }}
                        </span>
                      </td>
                      <td>
                        <button class="btn btn-primary btn-sm btn-tiny" @click="openRestoreModal(ref)">
                          Rest
                        </button>
                      </td>
                    </tr>
                  </tbody>
                </table>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- Side Utilities Panel: Prune backups -->
      <aside class="side-panel">
        <div class="card prune-box">
          <h3>Prune Backups</h3>
          <p class="description">Safely reclaim space by pruning old snapshots. At least one snapshot is always preserved.</p>
          
          <div class="form-group">
            <label for="prune-keep">Min Snapshots to Keep</label>
            <input id="prune-keep" type="number" v-model="pruneKeep" class="form-input" min="1" />
          </div>

          <div class="form-group">
            <label for="prune-age">Older Than (optional)</label>
            <input id="prune-age" type="text" v-model="pruneAge" class="form-input" placeholder="e.g. 30 days" />
          </div>

          <button class="btn btn-secondary w-100" @click="handlePrune" :disabled="pruning">
            <span v-if="pruning">Pruning...</span>
            <span v-else>🧹 Prune Snapshots</span>
          </button>

          <div v-if="pruneReport" class="prune-report-box">
            <p class="text-success">Success: Pruned {{ pruneReport.removed.length }} snapshots.</p>
            <p>Reclaimed: <strong>{{ formatBytes(pruneReport.reclaimedBytes) }}</strong></p>
          </div>
        </div>
      </aside>
    </div>

    <!-- Restore Overlay modal -->
    <div v-if="restoreRef" class="modal-overlay">
      <div class="modal-card card">
        <h3>Restore Reference</h3>
        <p class="restore-summary">Restoring <code>{{ restoreRef.branch }}</code> from snapshot <code>{{ activeSnapshotId }}</code></p>

        <div class="form-group">
          <label>Target Type</label>
          <div class="radio-group">
            <label class="radio-label">
              <input type="radio" value="branch" v-model="restoreTargetType" />
              Branch
            </label>
            <label class="radio-label">
              <input type="radio" value="tag" v-model="restoreTargetType" />
              Tag
            </label>
          </div>
        </div>

        <div class="form-group">
          <label for="restore-name">Restore Name</label>
          <input id="restore-name" type="text" v-model="restoreName" class="form-input" />
        </div>

        <div class="form-group">
          <label class="checkbox-container">
            <input type="checkbox" v-model="restoreForce" />
            <span class="checkmark"></span>
            Force overwrite if ref already exists
          </label>
        </div>

        <div v-if="restoreForce" class="warning-box">
          ⚠️ <strong>Force Overwrite Enabled:</strong> This will silently overwrite the existing local reference if names conflict!
        </div>

        <div class="modal-actions">
          <button class="btn btn-secondary" @click="restoreRef = null" :disabled="restoring">Cancel</button>
          <button class="btn btn-primary" @click="executeRestore" :disabled="restoring">
            <span v-if="restoring">Restoring...</span>
            <span v-else>Confirm Restore</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import { useReposStore } from '../stores/repos';
import {
  backupList,
  backupShow,
  backupVerify,
  backupPrune,
  restore,
  listenProgress,
  type ClientSnapshot,
  type ClientSnapshotDetail,
  type ClientVerifyReport,
  type ClientPruneReport
} from '../api/ipc';
import { formatLocalDateTime } from '../utils/date';

const store = useReposStore();

const selectedRepoId = ref(store.activeRepoId || '');
const snapshots = ref<ClientSnapshot[]>([]);
const loading = ref(false);

const activeSnapshotId = ref<string | null>(null);
const snapshotDetail = ref<ClientSnapshotDetail | null>(null);
const loadingDetails = ref(false);

// Verify State
const verifyingId = ref<string | null>(null);
const verifyResults = ref<ClientVerifyReport | null>(null);

// Prune State
const pruneKeep = ref(5);
const pruneAge = ref('');
const pruning = ref(false);
const pruneReport = ref<ClientPruneReport | null>(null);

// Restore State
const restoreRef = ref<any | null>(null);
const restoreTargetType = ref<'branch' | 'tag'>('branch');
const restoreName = ref('');
const restoreForce = ref(false);
const restoring = ref(false);

const formattedDate = (dateStr: string) => {
  return formatLocalDateTime(dateStr);
};

const formatBytes = (bytes: number) => {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

const triggerBadgeClass = (trig: string) => {
  if (trig === 'preDelete') return 'badge-danger';
  if (trig === 'scheduled') return 'badge-info';
  return 'badge-success';
};

const fetchSnapshots = async () => {
  if (!selectedRepoId.value) return;
  loading.value = true;
  try {
    snapshots.value = await backupList(selectedRepoId.value);
  } catch (err: any) {
    alert('Failed to load snapshots: ' + err.message);
  } finally {
    loading.value = false;
  }
};

const handleRepoChange = () => {
  if (selectedRepoId.value) {
    store.activeRepoId = selectedRepoId.value;
    activeSnapshotId.value = null;
    snapshotDetail.value = null;
    verifyResults.value = null;
    fetchSnapshots();
  }
};

const toggleSnapshot = async (id: string) => {
  if (activeSnapshotId.value === id) {
    activeSnapshotId.value = null;
    snapshotDetail.value = null;
    verifyResults.value = null;
  } else {
    activeSnapshotId.value = id;
    snapshotDetail.value = null;
    verifyResults.value = null;
    loadingDetails.value = true;
    try {
      snapshotDetail.value = await backupShow(id);
    } catch (err: any) {
      alert('Failed to load snapshot details: ' + err.message);
    } finally {
      loadingDetails.value = false;
    }
  }
};

const verifySnapshot = async (id: string) => {
  verifyingId.value = id;
  verifyResults.value = null;

  const taskId = `verify-${id}-${Date.now()}`;
  let unlistenFn: (() => void) | null = null;

  try {
    unlistenFn = await listenProgress((event) => {
      if (event.taskId === taskId) {
        // We can print verification progress if we want, or just let it finish
      }
    });

    const report = await backupVerify(id, taskId);
    verifyResults.value = {
      ...report,
      snapshotId: id
    };
  } catch (err: any) {
    alert('Verification failed: ' + err.message);
  } finally {
    verifyingId.value = null;
    if (unlistenFn) unlistenFn();
  }
};

const handlePrune = async () => {
  if (!selectedRepoId.value) return;
  pruning.value = true;
  pruneReport.value = null;
  try {
    const report = await backupPrune(
      selectedRepoId.value,
      pruneKeep.value,
      pruneAge.value.trim() || undefined
    );
    pruneReport.value = report;
    await fetchSnapshots();
  } catch (err: any) {
    alert('Prune failed: ' + err.message);
  } finally {
    pruning.value = false;
  }
};

const openRestoreModal = (refItem: any) => {
  restoreRef.value = refItem;
  restoreTargetType.value = 'branch';
  restoreName.value = refItem.branch;
  restoreForce.value = false;
};

const executeRestore = async () => {
  if (!activeSnapshotId.value || !restoreRef.value) return;
  restoring.value = true;
  try {
    const outcome = await restore(activeSnapshotId.value, {
      refName: restoreRef.value.branch,
      targetType: restoreTargetType.value,
      newName: restoreName.value.trim() !== restoreRef.value.branch ? restoreName.value.trim() : undefined,
      force: restoreForce.value,
      originalRef: restoreRef.value.originalRef
    });
    alert(`Successfully restored reference ${outcome.restored} as a ${outcome.as}! Tip Commit SHA: ${outcome.sha.substring(0, 7)}`);
    restoreRef.value = null;
  } catch (err: any) {
    alert('Restore failed: ' + err.message);
  } finally {
    restoring.value = false;
  }
};

watch(() => store.activeRepoId, (newId) => {
  if (newId) {
    selectedRepoId.value = newId;
    fetchSnapshots();
  }
});

onMounted(() => {
  if (selectedRepoId.value) {
    fetchSnapshots();
  }
});
</script>

<style scoped>
.backups-container {
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

.backups-layout {
  display: flex;
  gap: var(--spacing-md);
  flex-grow: 1;
  overflow: hidden;
}

/* Snapshots List */
.snapshots-list {
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

.loading-state, .empty-state {
  display: flex;
  justify-content: center;
  align-items: center;
  padding: var(--spacing-xl);
  color: var(--muted);
  flex-grow: 1;
}

.loading-state-sm {
  display: flex;
  justify-content: center;
  align-items: center;
  padding: var(--spacing-md);
  color: var(--muted);
}

.spinner {
  width: 24px;
  height: 24px;
  border: 3px solid var(--border);
  border-top-color: var(--primary);
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-right: var(--spacing-sm);
}

.spinner-sm {
  width: 14px;
  height: 14px;
  border-width: 2px;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.snapshots-grid {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
  overflow-y: auto;
  flex-grow: 1;
  padding-right: 4px;
}

.snap-card {
  display: flex;
  flex-direction: column;
  padding: 0;
  overflow: hidden;
  transition: border-color var(--transition-fast);
}

.snap-card:hover {
  border-color: var(--muted);
}

.snap-active {
  border-color: var(--primary) !important;
}

.snap-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-sm) var(--spacing-md);
  cursor: pointer;
  background-color: var(--surface-raised);
}

.snap-meta {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.snap-id code {
  background-color: var(--border);
  padding: 2px 6px;
  border-radius: var(--radius-xs);
  color: var(--primary);
  font-family: var(--font-mono);
}

.snap-date {
  font-size: 12px;
  color: var(--muted);
}

.snap-right {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  font-size: 13px;
  color: var(--on-surface);
}

.arrow {
  display: inline-block;
  font-size: 10px;
  color: var(--muted);
  transition: transform var(--transition-fast);
}

.arrow-down {
  transform: rotate(90deg);
}

.snap-details {
  border-top: 1px solid var(--border);
  padding: var(--spacing-md);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
  background-color: var(--surface);
}

.snap-actions {
  display: flex;
  gap: var(--spacing-sm);
}

.verify-results-box {
  padding: var(--spacing-sm);
  border-radius: var(--radius-xs);
  font-size: 12px;
}

.verify-ok {
  background-color: rgba(152, 195, 121, 0.08);
  border: 1px solid rgba(152, 195, 121, 0.2);
  color: var(--success);
}

.verify-failed {
  background-color: rgba(224, 108, 117, 0.08);
  border: 1px solid rgba(224, 108, 117, 0.2);
  color: var(--danger);
}

.verify-results-box ul {
  padding-left: var(--spacing-md);
  margin-top: 4px;
}

.refs-table-wrapper {
  border: 1px solid var(--border);
  border-radius: var(--radius-xs);
  max-height: 250px;
  overflow-y: auto;
}

.refs-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12px;
}

.refs-table th, .refs-table td {
  padding: 6px var(--spacing-sm);
  border-bottom: 1px solid var(--border);
  text-align: left;
}

.refs-table th {
  background-color: var(--surface-variant);
  color: var(--on-surface-strong);
  font-weight: 600;
  position: sticky;
  top: 0;
  z-index: 1;
}

.branch-name {
  font-family: var(--font-mono);
  color: var(--on-surface-strong);
}

.upstream-lbl {
  display: block;
  font-size: 10px;
  color: var(--muted);
}

.btn-tiny {
  font-size: 10px;
  padding: 1px var(--spacing-xs);
}

/* Side Panel */
.side-panel {
  width: 260px;
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
  flex-shrink: 0;
}

.prune-box {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
}

.prune-box h3 {
  font-size: 14px;
  color: var(--on-surface-strong);
}

.description {
  font-size: 12px;
  color: var(--muted);
  line-height: 1.4;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.form-group label {
  font-size: 11px;
  color: var(--on-surface);
}

.form-input {
  background-color: var(--surface-raised);
  color: var(--on-surface);
  border: 1px solid var(--border);
  padding: var(--spacing-xs) var(--spacing-sm);
  border-radius: var(--radius-xs);
  outline: none;
  font-size: 13px;
}

.form-input:focus {
  border-color: var(--primary);
}

.w-100 {
  width: 100%;
}

.prune-report-box {
  background-color: var(--surface-variant);
  padding: var(--spacing-sm);
  border-radius: var(--radius-xs);
  font-size: 12px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

/* Modal Overlay */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: var(--overlay);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.modal-card {
  width: 90%;
  max-width: 440px;
  background-color: var(--surface-raised);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
}

.restore-summary {
  font-size: 13px;
  color: var(--muted);
}

.radio-group {
  display: flex;
  gap: var(--spacing-md);
}

.radio-label {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  font-size: 13px;
  cursor: pointer;
}

.checkbox-container {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  font-size: 13px;
  cursor: pointer;
}

.warning-box {
  background-color: rgba(229, 192, 123, 0.08);
  border: 1px solid rgba(229, 192, 123, 0.2);
  color: var(--warning);
  padding: var(--spacing-sm);
  border-radius: var(--radius-xs);
  font-size: 12px;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--spacing-sm);
  border-top: 1px solid var(--border);
  padding-top: var(--spacing-sm);
}
</style>
