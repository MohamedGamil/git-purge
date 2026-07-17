<template>
  <div class="cleanup-container">
    <header class="view-header">
      <div>
        <h1>Repository Cleanup</h1>
        <p class="subtitle" v-if="store.activeRepoDetail">
          Run automated stale branch cleanup for <strong>{{ store.activeRepoDetail.name }}</strong>
        </p>
        <p class="subtitle" v-else>Select a repository to run automated cleanup audits.</p>
      </div>

      <div class="repo-selector">
        <label for="repo-select">Repository: </label>
        <select id="repo-select" v-model="selectedRepoId" @change="handleRepoChange" :disabled="store.loading || loadingPlan || isExecuting">
          <option value="" disabled>-- Select Repository --</option>
          <option v-for="repo in store.repos" :key="repo.id" :value="repo.id">
            {{ repo.name }}
          </option>
        </select>
      </div>
    </header>

    <div v-if="!store.activeRepoId" class="select-prompt card">
      <p>Please select a repository from the dropdown above to configure cleanup.</p>
    </div>

    <div v-else class="cleanup-layout">
      <!-- Left sidebar: Cleanup Options -->
      <aside class="controls-panel card">
        <div class="control-group">
          <h3>Cleanup Scope</h3>
          
          <div class="filter-item">
            <label for="action-kind">Action Mode</label>
            <select id="action-kind" v-model="actionKind" class="form-input" :disabled="store.loading || loadingPlan || isExecuting">
              <option value="delete">Purge/Delete Mode</option>
              <option value="archive">Archive Mode</option>
            </select>
          </div>

          <div class="filter-item">
            <label for="age-override">Age Threshold Override</label>
            <input
              id="age-override"
              type="text"
              v-model="ageOverride"
              placeholder="e.g. 90 days ago, 1 year ago"
              class="form-input"
              :disabled="store.loading || loadingPlan || isExecuting"
            />
            <span class="input-hint">Leave blank to use default policy.</span>
          </div>

          <!-- Delete Mode Options -->
          <div class="filter-item checkbox-group" v-if="actionKind === 'delete'">
            <label class="checkbox-container">
              <input type="checkbox" v-model="mergedOnly" :disabled="store.loading || loadingPlan || isExecuting" />
              <span class="checkmark"></span>
              Merged branches only
            </label>
          </div>

          <div class="filter-item checkbox-group" v-if="actionKind === 'delete'">
            <label class="checkbox-container">
              <input type="checkbox" v-model="includeUnmerged" :disabled="store.loading || loadingPlan || isExecuting" />
              <span class="checkmark"></span>
              Include unmerged branches
            </label>
          </div>

          <!-- Archive Mode Merge Strategy Selector -->
          <div class="filter-item merge-strategy-wrapper" v-if="actionKind === 'archive' && hasUnmergedBranches">
            <label for="merge-strategy">Merge / Archive Strategy</label>
            <select id="merge-strategy" v-model="mergeStrategy" class="form-input" :disabled="store.loading || loadingPlan || isExecuting">
              <option value="skip">Skip Unmerged (Safe)</option>
              <option value="force">Force Archive Unmerged (Dangerous)</option>
              <option value="merge-first">Fast-Forward Merge, then Archive</option>
            </select>
            <span class="input-hint">Unmerged branches were detected in this repository.</span>
          </div>

          <!-- Archive Destination Branch -->
          <div class="filter-item" v-if="actionKind === 'archive'">
            <label for="archive-target">Archive Destination Branch</label>
            <input
              id="archive-target"
              type="text"
              v-model="archiveTargetBranch"
              class="form-input"
              :disabled="store.loading || loadingPlan || isExecuting"
              placeholder="main-legacy"
            />
            <span class="input-hint">Target branch where archived commits are stored.</span>
          </div>

          <!-- Archive Merge Strategy (Ours vs Theirs) -->
          <div class="filter-item" v-if="actionKind === 'archive'">
            <label for="archive-strategy">Git Merge Strategy</label>
            <select id="archive-strategy" v-model="archiveStrategy" class="form-input" :disabled="store.loading || loadingPlan || isExecuting">
              <option value="ours">Ours (Prefer Legacy Content)</option>
              <option value="theirs">Theirs (Prefer Incoming Content)</option>
            </select>
            <span class="input-hint">Conflict resolution preference.</span>
          </div>

          <button class="btn btn-primary w-100" @click="generatePlan" :disabled="store.loading || loadingPlan || isExecuting">
            <span v-if="loadingPlan">Analyzing...</span>
            <span v-else><Search class="lucide-icon" style="margin-right: 4px;" /> Generate Cleanup Plan</span>
          </button>
        </div>
      </aside>

      <!-- Main Plan Area -->
      <section class="plan-details-area">
        <!-- 1. LOADING DRY-RUN PLAN -->
        <div v-if="loadingPlan" class="loading-state card">
          <span class="spinner"></span>
          <p>Generating cleanup plan. Evaluating safety policies...</p>
        </div>

        <!-- 2. ERROR DISPLAY -->
        <div v-else-if="planError" class="error-state card">
          <h3>Plan Generation Failed</h3>
          <p class="error-msg">{{ planError }}</p>
          <button class="btn btn-primary" @click="generatePlan">Retry Plan Generation</button>
        </div>

        <!-- 3. PLAN DISPLAY & EXECUTION PANEL -->
        <div v-else class="plan-results-layout">
          <div class="plan-details card">
            <div class="details-header">
              <h2>Proposed Actions ({{ planResult?.actions.length || 0 }})</h2>
              <span class="action-badge" :class="actionKind === 'archive' ? 'badge-info' : 'badge-danger'">
                {{ actionKind === 'archive' ? 'Archive Mode' : 'Delete/Purge Mode' }}
              </span>
            </div>

            <div v-if="!planResult || planResult.actions.length === 0" class="empty-plan">
              <p>No actions proposed in the plan. No branches match the selected cleanup scope.</p>
            </div>

            <div v-else class="actions-list">
              <div v-for="act in planResult.actions" :key="act.refName" class="action-card" :class="{ 'destructive-card': act.destructive }">
                <div class="action-card-header">
                  <span class="action-type" :class="act.action === 'archive' ? 'text-archive' : 'text-delete'">
                    <Archive v-if="act.action === 'archive'" class="lucide-icon" style="margin-right: 4px;" />
                    <Trash2 v-else class="lucide-icon" style="margin-right: 4px;" />
                    {{ act.action === 'archive' ? 'Archive' : 'Delete' }}
                  </span>
                  <span class="action-branch"><code>{{ act.refName }}</code></span>
                  <span v-if="act.destructive" class="badge badge-danger">Unmerged Branch</span>
                </div>
                <div class="action-reason">
                  <strong>Reason:</strong> {{ act.reason }}
                </div>
                <div class="action-meta">
                  <span class="badge badge-secondary badge-tiny">{{ act.classification.locality }}</span>
                  <span class="badge badge-secondary badge-tiny">{{ act.classification.freshness }}</span>
                  <span class="badge badge-secondary badge-tiny">{{ act.classification.merge }}</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Execution control panel -->
          <aside class="execution-panel">
            <!-- Progress overlay / card -->
            <div v-if="isExecuting" class="execution-progress-card card">
              <h3>Executing Plan...</h3>
              <div class="progress-bar-bg">
                <div class="progress-bar-fill" :style="{ width: execProgress + '%' }"></div>
              </div>
              <div class="progress-meta">
                <span class="progress-pct">{{ execProgress }}%</span>
                <span class="progress-msg">{{ execProgressMessage }}</span>
              </div>

              <!-- Live Operations Log Panel -->
              <div class="live-ops-log card">
                <h4>Operation Log</h4>
                <div class="ops-log-content">
                  <div v-for="(log, index) in executionLogs" :key="index" class="log-line text-sm code-font">
                    {{ log }}
                  </div>
                </div>
              </div>

              <button class="btn btn-danger w-100 cancel-btn" @click="handleCancel">
                <OctagonAlert class="lucide-icon" style="margin-right: 4px;" /> Cancel Operations
              </button>
            </div>

            <!-- Run Report / Success Display -->
            <div v-else-if="runReport" class="run-report-card card">
              <h3 class="success-header"><PartyPopper class="lucide-icon" style="margin-right: 6px;" /> Cleanup Complete</h3>
              <div class="report-stats">
                <div class="report-stat">
                  <span class="stat-lbl">Attempted</span>
                  <span class="stat-val">{{ runReport.attempted }}</span>
                </div>
                <div class="report-stat">
                  <span class="stat-lbl text-success">Succeeded</span>
                  <span class="stat-val text-success">{{ runReport.succeeded }}</span>
                </div>
                <div class="report-stat">
                  <span class="stat-lbl text-danger">Failed</span>
                  <span class="stat-val text-danger">{{ runReport.failed }}</span>
                </div>
              </div>

              <div class="report-backup" v-if="runReport.snapshotId">
                <h4>Safety Net Snapshot ID</h4>
                <div class="snapshot-id-box">
                  <code>{{ runReport.snapshotId }}</code>
                </div>
                <p class="backup-hint">You can use this ID in the Backups screen to restore any branch if needed.</p>
              </div>

              <button class="btn btn-primary w-100" @click="resetFlow">
                Done & Return to Cleanup
              </button>
            </div>

            <!-- Regular Execution Setup -->
            <div v-else class="execution-setup card">
              <h3>Safety Safeguards</h3>

              <!-- Dynamic Safeguards Description -->
              <div class="safeguards-description">
                <div v-if="actionKind === 'delete'" class="mode-info delete-info">
                  <p><Trash2 class="lucide-icon color-danger" style="margin-right: 4px;" /> <strong>Delete/Purge Mode:</strong> You are about to permanently delete matching branches from the remote and local systems. This action cannot be undone.</p>
                </div>
                <div v-else class="mode-info archive-info">
                  <p><Archive class="lucide-icon color-primary" style="margin-right: 4px;" /> <strong>Archive Mode:</strong> Selected branches will be safely archived (renamed/prefixed to <code>archive/</code> or backed up in bare mirrors). The original references will be deleted from the active tracking list.</p>
                </div>
              </div>

              <div class="safety-options">
                <label class="checkbox-container select-none">
                  <input type="checkbox" v-model="noBackup" :disabled="store.loading || loadingPlan || isExecuting" />
                  <span class="checkmark"></span>
                  <span v-if="actionKind === 'delete'">Disable pre-deletion backup snapshot</span>
                  <span v-else>Disable pre-archival backup snapshot</span>
                </label>
                <div v-if="noBackup" class="warning-box">
                  <TriangleAlert class="lucide-icon color-danger" style="margin-right: 4px;" /> <strong>Caution:</strong> Proceeding without a backup snapshot is highly dangerous and cannot be undone!
                </div>
              </div>

              <!-- Unmerged Destructive Confirmation (SAFE-02) -->
              <div v-if="hasDestructiveActions" class="destructive-confirmation">
                <label for="confirm-token">
                  <TriangleAlert class="lucide-icon color-danger" style="margin-right: 4px;" /> This plan contains unmerged branches. Type <strong>DELETE</strong> to confirm:
                </label>
                <input
                  id="confirm-token"
                  type="text"
                  v-model="confirmToken"
                  placeholder="Type DELETE here..."
                  class="form-input w-100 confirm-input"
                  :disabled="store.loading || loadingPlan || isExecuting"
                />
              </div>

              <!-- Dynamic CTA Button based on mode -->
              <button
                v-if="actionKind === 'delete'"
                class="btn btn-danger w-100 execute-btn"
                :disabled="!canExecute || store.loading || loadingPlan || isExecuting"
                @click="executePlan"
              >
                <Trash2 class="lucide-icon" style="margin-right: 4px;" /> Execute Destructive Purge
              </button>
              <button
                v-else
                class="btn btn-primary w-100 execute-btn"
                :disabled="!canExecute || store.loading || loadingPlan || isExecuting"
                @click="executePlan"
              >
                <Archive class="lucide-icon" style="margin-right: 4px;" /> Execute Branch Archival
              </button>
              
              <p class="dry-run-hint">Changes will only be applied to the repository after you click the button above.</p>
            </div>
          </aside>
        </div>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { useReposStore } from '../stores/repos';
import { useBranchesStore } from '../stores/branches';
import { useToastStore } from '../stores/toast';
import { Search, Archive, Trash2, OctagonAlert, PartyPopper, TriangleAlert } from '@lucide/vue';

const toastStore = useToastStore();
import {
  type ClientPlan,
  type ClientRunReport
} from '../api/ipc';

const store = useReposStore();
const branchesStore = useBranchesStore();

const selectedRepoId = ref(store.activeRepoId || '');
const actionKind = ref<'delete' | 'archive'>('delete');
const ageOverride = ref('');
const mergedOnly = ref(true);
const includeUnmerged = ref(false);

// Archive Mode Merge Strategy
const mergeStrategy = ref<'skip' | 'force' | 'merge-first'>('skip');
const archiveTargetBranch = ref('main-legacy');
const archiveStrategy = ref<'ours' | 'theirs'>('ours');

const hasUnmergedBranches = computed(() => {
  return store.branches.some(b => b.classification.merge === 'unmerged');
});

// Watch Strategy changes
watch(mergeStrategy, (newStrategy) => {
  if (actionKind.value === 'archive') {
    if (newStrategy === 'skip') {
      mergedOnly.value = true;
      includeUnmerged.value = false;
    } else {
      mergedOnly.value = false;
      includeUnmerged.value = true;
    }
    generatePlan();
  }
});

// Watch Mode changes
watch(actionKind, (newKind) => {
  if (newKind === 'archive') {
    mergeStrategy.value = 'skip';
    mergedOnly.value = true;
    includeUnmerged.value = false;
  }
  generatePlan();
});

const loadingPlan = computed(() => branchesStore.loadingPlan);
const planError = computed(() => branchesStore.planError);
const planResult = computed(() => branchesStore.planResult);

// Safety configurations
const noBackup = ref(false);
const confirmToken = ref('');

// Execution state
const isExecuting = computed(() => branchesStore.isExecuting);
const execTaskId = computed(() => branchesStore.execTaskId);
const execProgress = computed(() => branchesStore.execProgress);
const execProgressMessage = computed(() => branchesStore.execProgressMessage);
const runReport = computed(() => branchesStore.runReport);
const executionLogs = ref<string[]>([]);

// Watch progress message to append to logs
watch(execProgressMessage, (msg) => {
  if (msg) {
    executionLogs.value.push(msg);
    setTimeout(() => {
      const container = document.querySelector('.ops-log-content');
      if (container) {
        container.scrollTop = container.scrollHeight;
      }
    }, 50);
  }
});

const hasDestructiveActions = computed(() => {
  return planResult.value?.actions.some(a => a.destructive) || false;
});

const canExecute = computed(() => {
  if (!planResult.value || planResult.value.actions.length === 0) return false;
  if (hasDestructiveActions.value) {
    return confirmToken.value.trim() === 'DELETE';
  }
  return true;
});

const handleRepoChange = () => {
  if (selectedRepoId.value) {
    store.selectRepo(selectedRepoId.value);
    branchesStore.resetPlanAndReport();
  }
};

const generatePlan = async () => {
  if (!selectedRepoId.value) return;

  try {
    await branchesStore.generatePlan(selectedRepoId.value, {
      kind: actionKind.value,
      age: ageOverride.value.trim() || undefined,
      merged: mergedOnly.value,
      includeUnmerged: includeUnmerged.value
    });
  } catch (err: any) {
    console.error('Plan generation failed:', err);
  }
};

const executePlan = async () => {
  if (!canExecute.value || !selectedRepoId.value || !planResult.value) return;

  executionLogs.value = [];
  const execOpts = {
    noBackup: noBackup.value,
    confirmedToken: hasDestructiveActions.value ? confirmToken.value : undefined,
    targetBranch: archiveTargetBranch.value.trim() || 'main-legacy',
    strategy: archiveStrategy.value
  };

  try {
    if (actionKind.value === 'archive') {
      await branchesStore.executeArchive(selectedRepoId.value, planResult.value, execOpts);
    } else {
      await branchesStore.executeDelete(selectedRepoId.value, planResult.value, execOpts);
    }
    confirmToken.value = '';

    // Refresh repo lists and active repo details
    await store.fetchRepos();
    if (store.activeRepoId === selectedRepoId.value) {
      await store.selectRepo(selectedRepoId.value);
    }
  } catch (err: any) {
    toastStore.error('Cleanup failed: ' + (err?.message || err));
  }
};

const handleCancel = async () => {
  await branchesStore.cancelActiveTask();
};

const resetFlow = () => {
  branchesStore.resetPlanAndReport();
};

watch(() => store.activeRepoId, (newId) => {
  if (newId) {
    selectedRepoId.value = newId;
    branchesStore.resetPlanAndReport();
  }
});

onMounted(() => {
  if (selectedRepoId.value) {
    generatePlan();
  }
});
</script>

<style scoped>
.cleanup-container {
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

.cleanup-layout {
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

.control-group {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
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

.filter-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.filter-item label {
  font-size: 11px;
  color: var(--on-surface);
}

.form-input {
  background-color: var(--surface-raised);
  color: var(--on-surface);
  border: 1px solid var(--border);
  padding: var(--spacing-sm);
  border-radius: var(--radius-xs);
  outline: none;
  font-size: 12px;
}

.form-input:focus {
  border-color: var(--primary);
}

.input-hint {
  font-size: 10px;
  color: var(--muted);
  margin-top: 2px;
  line-height: 1.4;
}

.checkbox-group {
  margin-top: 4px;
}

.checkbox-container {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  font-size: 13px;
  color: var(--on-surface);
  cursor: pointer;
  user-select: none;
}

.w-100 {
  width: 100%;
}

/* Plan Details Area */
.plan-details-area {
  flex-grow: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.loading-state, .error-state {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  flex-grow: 1;
  gap: var(--spacing-md);
  color: var(--muted);
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
  background-color: rgba(224, 108, 117, 0.05);
  padding: var(--spacing-sm);
  border-radius: var(--radius-xs);
  border: 1px solid rgba(224, 108, 117, 0.2);
}

.plan-results-layout {
  display: flex;
  gap: var(--spacing-md);
  flex-grow: 1;
  overflow: hidden;
}

.plan-details {
  flex-grow: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.details-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-sm);
  flex-shrink: 0;
}

.details-header h2 {
  font-size: 16px;
  color: var(--on-surface-strong);
}

.empty-plan {
  display: flex;
  justify-content: center;
  align-items: center;
  flex-grow: 1;
  color: var(--muted);
}

.actions-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
  overflow-y: auto;
  flex-grow: 1;
  padding-right: 4px;
}

.action-card {
  background-color: var(--surface-raised);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: var(--spacing-sm) var(--spacing-md);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.destructive-card {
  border-left: 4px solid var(--danger);
  background-color: rgba(224, 108, 117, 0.02);
}

.action-card-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.action-type {
  font-weight: 600;
  text-transform: uppercase;
  font-size: 11px;
}

.text-delete {
  color: var(--danger);
}

.text-archive {
  color: var(--info);
}

.action-branch {
  font-family: var(--font-mono);
  color: var(--on-surface-strong);
  font-weight: 500;
}

.action-reason {
  font-size: 12px;
  color: var(--on-surface);
}

.action-meta {
  display: flex;
  gap: 4px;
}

.badge-tiny {
  padding: 1px 4px;
  font-size: 9px;
}

/* Execution Panel */
.execution-panel {
  width: 320px;
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
  flex-shrink: 0;
}

.execution-setup, .execution-progress-card, .run-report-card {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
  height: max-content;
}

.execution-setup h3, .execution-progress-card h3, .run-report-card h3 {
  font-size: 14px;
  color: var(--on-surface-strong);
  border-bottom: 1px solid var(--border);
  padding-bottom: var(--spacing-xs);
}

.safety-options {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.warning-box {
  background-color: rgba(229, 192, 123, 0.08);
  border: 1px solid rgba(229, 192, 123, 0.2);
  color: var(--warning);
  padding: var(--spacing-sm);
  border-radius: var(--radius-xs);
  font-size: 12px;
}

.destructive-confirmation {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.destructive-confirmation label {
  font-size: 12px;
  color: var(--on-surface);
  line-height: 1.4;
}

.confirm-input {
  font-size: 14px;
  font-weight: 600;
  text-align: center;
  letter-spacing: 1px;
}

.dry-run-hint {
  font-size: 11px;
  color: var(--muted);
  text-align: center;
}

/* Progress and Report */
.progress-bar-bg {
  width: 100%;
  height: 8px;
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
  font-size: 12px;
}

.progress-msg {
  color: var(--muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 220px;
}

.success-header {
  color: var(--success) !important;
}

.report-stats {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--spacing-sm);
  text-align: center;
}

.report-stat {
  display: flex;
  flex-direction: column;
  background-color: var(--surface-variant);
  border-radius: var(--radius-xs);
  padding: var(--spacing-sm) 0;
}

.stat-lbl {
  font-size: 11px;
  color: var(--muted);
  text-transform: uppercase;
}

.stat-val {
  font-size: 20px;
  font-weight: 700;
  color: var(--on-surface-strong);
}

.report-backup h4 {
  font-size: 12px;
  color: var(--on-surface);
  margin-bottom: var(--spacing-xs);
}

.snapshot-id-box {
  background-color: var(--surface-variant);
  border: 1px solid var(--border);
  padding: var(--spacing-sm);
  border-radius: var(--radius-xs);
  font-family: var(--font-mono);
  font-size: 13px;
  color: var(--primary);
  text-align: center;
  user-select: text;
}

.backup-hint {
  font-size: 11px;
  color: var(--muted);
  margin-top: 4px;
  line-height: 1.4;
}

.live-ops-log {
  margin: var(--spacing-md) 0;
  padding: var(--spacing-sm);
  background-color: rgba(0, 0, 0, 0.2);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.live-ops-log h4 {
  font-size: 11px;
  text-transform: uppercase;
  color: var(--muted);
  margin: 0 0 var(--spacing-xs) 0;
}

.ops-log-content {
  max-height: 180px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding-right: 4px;
}

.log-line {
  color: var(--on-surface);
  line-height: 1.4;
  word-break: break-all;
  border-left: 2px solid var(--primary);
  padding-left: 6px;
}
</style>
