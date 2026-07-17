<template>
  <div class="plan-container">
    <header class="view-header">
      <div>
        <h1>Action Plan Review</h1>
        <p class="subtitle" v-if="activeRepo">
          Previewing changes for repository <strong>{{ activeRepo.name }}</strong>
        </p>
      </div>
      <button class="btn btn-secondary btn-sm" @click="goBack" :disabled="isExecuting">
        <X class="lucide-icon" style="margin-right: 4px;" /> Abort & Go Back
      </button>
    </header>

    <!-- 1. LOADING DRY-RUN PLAN -->
    <div v-if="loadingPlan" class="loading-state card">
      <span class="spinner"></span>
      <p>Generating dry-run plan. Evaluating safety policies...</p>
    </div>

    <!-- 2. ERROR DISPLAY -->
    <div v-else-if="planError" class="error-state card">
      <h3>Plan Generation Failed</h3>
      <p class="error-msg">{{ planError }}</p>
      <button class="btn btn-primary" @click="generatePlan">Retry Plan Generation</button>
    </div>

    <!-- 3. PLAN DISPLAY & EXECUTION PANEL -->
    <div v-else class="plan-layout">
      <div class="plan-details card">
        <div class="details-header">
          <h2>Proposed Actions ({{ planResult?.actions.length || 0 }})</h2>
          <span class="action-badge" :class="actionKind === 'archive' ? 'badge-info' : 'badge-danger'">
            {{ actionKind === 'archive' ? 'Archive Mode' : 'Delete/Purge Mode' }}
          </span>
        </div>

        <div v-if="planResult?.actions.length === 0" class="empty-plan">
          <p>No actions proposed in the plan. All selected branches might already satisfy the desired state.</p>
        </div>

        <div v-else class="actions-list">
          <div v-for="act in planResult?.actions" :key="act.refName" class="action-card" :class="{ 'destructive-card': act.destructive }">
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
          <button class="btn btn-danger w-100 cancel-btn" @click="handleCancel">
            <OctagonAlert class="lucide-icon" style="margin-right: 4px;" /> Cancel/Abort Operations
          </button>
        </div>

        <!-- Run Report / Success Display -->
        <div v-else-if="runReport" class="run-report-card card">
          <h3 class="success-header"><PartyPopper class="lucide-icon" style="margin-right: 6px;" /> Execution Complete</h3>
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

          <button class="btn btn-primary w-100" @click="finishFlow">
            Done & Return to Explorer
          </button>
        </div>

        <!-- Regular Execution Setup -->
        <div v-else class="execution-setup card">
          <h3>Safety Safeguards</h3>

          <!-- Remote Actions Warning -->
          <div v-if="hasRemoteActions" class="warning-box" style="margin-top: var(--spacing-xs); margin-bottom: var(--spacing-sm);">
            <TriangleAlert class="lucide-icon color-warning" style="margin-right: 4px;" /> <strong>Caution:</strong> This plan contains <strong>remote</strong> branch deletions. Deleting remote branches will permanently delete refs on the remote Git server.
          </div>

          <div class="safety-options">
            <label class="checkbox-container select-none">
              <input type="checkbox" v-model="noBackup" />
              <span class="checkmark"></span>
              Disable pre-deletion backup snapshot
            </label>
            <div v-if="noBackup" class="warning-box">
              <TriangleAlert class="lucide-icon color-danger" style="margin-right: 4px;" /> <strong>Caution:</strong> Proceeding without a backup snapshot is dangerous and cannot be undone!
            </div>
          </div>

          <!-- Unmerged Destructive Confirmation (SAFE-02 / SAFETY) -->
          <div v-if="hasDestructiveActions" class="destructive-confirmation">
            <label for="confirm-token">
              <TriangleAlert class="lucide-icon color-danger" style="margin-right: 4px;" /> This plan contains unmerged branches. Type <strong>DELETE</strong> to confirm destruction:
            </label>
            <input
              id="confirm-token"
              type="text"
              v-model="confirmToken"
              placeholder="Type DELETE here..."
              class="form-input w-100 confirm-input"
            />
          </div>

          <button
            class="btn btn-danger w-100 execute-btn"
            :disabled="!canExecute"
            @click="executePlan"
          >
            <Rocket class="lucide-icon" style="margin-right: 4px;" /> Execute Plan (Write Changes)
          </button>
          <p class="dry-run-hint">Changes will only be applied after you click the button above.</p>
        </div>
      </aside>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useReposStore } from '../stores/repos';
import { useBranchesStore } from '../stores/branches';
import { X, Archive, Trash2, OctagonAlert, PartyPopper, TriangleAlert, Rocket } from '@lucide/vue';
import {
  type ClientPlan,
  type ClientRunReport
} from '../api/ipc';

const route = useRoute();
const router = useRouter();
const store = useReposStore();
const branchesStore = useBranchesStore();

const repoId = computed(() => route.query.repoId as string);
const actionKind = computed(() => (route.query.actionKind || 'delete') as 'delete' | 'archive');
const refsParam = computed(() => (route.query.refs as string) || '');

const loadingPlan = computed(() => branchesStore.loadingPlan);
const planError = computed(() => branchesStore.planError);
const planResult = computed(() => branchesStore.planResult);

// Safety configuration
const noBackup = ref(false);
const confirmToken = ref('');

// Execution state
const isExecuting = computed(() => branchesStore.isExecuting);
const execTaskId = computed(() => branchesStore.execTaskId);
const execProgress = computed(() => branchesStore.execProgress);
const execProgressMessage = computed(() => branchesStore.execProgressMessage);
const runReport = computed(() => branchesStore.runReport);

const activeRepo = computed(() => {
  return store.repos.find(r => r.id === repoId.value);
});

const hasDestructiveActions = computed(() => {
  return planResult.value?.actions.some(a => a.destructive) || false;
});

const hasRemoteActions = computed(() => {
  return planResult.value?.actions.some(a => a.classification.locality === 'remote') || false;
});

const canExecute = computed(() => {
  if (!planResult.value || planResult.value.actions.length === 0) return false;
  if (hasDestructiveActions.value) {
    return confirmToken.value.trim() === 'DELETE';
  }
  return true;
});

const generatePlan = async () => {
  if (!repoId.value || !refsParam.value) {
    branchesStore.planError = 'Missing repository or branches parameters.';
    return;
  }

  try {
    const refsList = refsParam.value.split(',');
    await branchesStore.generatePlan(repoId.value, {
      kind: actionKind.value,
      refs: refsList
    });
  } catch (err: any) {
    console.error('Plan generation failed:', err);
  }
};

const executePlan = async () => {
  if (!canExecute.value || !repoId.value || !planResult.value) return;

  const execOpts = {
    noBackup: noBackup.value,
    confirmedToken: hasDestructiveActions.value ? confirmToken.value : undefined
  };

  try {
    if (actionKind.value === 'archive') {
      await branchesStore.executeArchive(repoId.value, planResult.value, execOpts);
    } else {
      await branchesStore.executeDelete(repoId.value, planResult.value, execOpts);
    }

    // Refresh repo info and branches list in the background
    await store.fetchRepos();
    if (store.activeRepoId === repoId.value) {
      await store.runScan(repoId.value, { includeRemote: true });
    }
  } catch (err: any) {
    alert('Execution failed: ' + (err?.message || err));
  }
};

const handleCancel = async () => {
  await branchesStore.cancelActiveTask();
};

const goBack = () => {
  router.push('/branches');
};

const finishFlow = () => {
  router.push('/branches');
};

onMounted(() => {
  branchesStore.resetPlanAndReport();
  generatePlan();
});
</script>

<style scoped>
.plan-container {
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

.plan-layout {
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
  transition: border-color var(--transition-fast);
}

.action-card:hover {
  border-color: var(--muted);
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

.checkbox-container {
  display: flex;
  align-items: flex-start;
  gap: var(--spacing-sm);
  font-size: 13px;
  color: var(--on-surface);
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

.w-100 {
  width: 100%;
}

.form-input {
  background-color: var(--surface-raised);
  color: var(--on-surface);
  border: 1px solid var(--border);
  padding: var(--spacing-sm);
  border-radius: var(--radius-xs);
  outline: none;
}

.form-input:focus {
  border-color: var(--primary);
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
</style>
