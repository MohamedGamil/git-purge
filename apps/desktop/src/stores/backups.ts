import { defineStore } from 'pinia';
import {
  backupList,
  backupShow,
  backupCreate,
  backupVerify,
  backupPrune,
  restore,
  cancel,
  listenProgress,
  type ClientSnapshot,
  type ClientSnapshotDetail,
  type ClientBackupOptions,
  type ClientRestoreSpec,
  type ClientVerifyReport,
  type ClientPruneReport
} from '../api/ipc';

export const useBackupsStore = defineStore('backups', {
  state: () => ({
    snapshots: [] as ClientSnapshot[],
    activeSnapshot: null as ClientSnapshotDetail | null,
    loading: false,
    error: null as string | null,
    
    // Verifying State
    isVerifying: false,
    verifyProgress: 0,
    verifyProgressMessage: '',
    activeVerifyTaskId: null as string | null,
    verifyReport: null as ClientVerifyReport | null,

    // Pruning State
    isPruning: false,
    pruneReport: null as ClientPruneReport | null,

    // Restoring State
    isRestoring: false,

    // Creating State
    isBackingUp: false,
    backupProgress: 0,
    backupProgressMessage: '',
    activeBackupTaskId: null as string | null,
  }),

  actions: {
    async fetchSnapshots(repoId: string) {
      this.loading = true;
      this.error = null;
      try {
        const list = await backupList(repoId);
        this.snapshots = list.sort((a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime());
        return this.snapshots;
      } catch (err: any) {
        this.error = err?.message || 'Failed to fetch backups';
        throw err;
      } finally {
        this.loading = false;
      }
    },

    async fetchSnapshotDetail(snapshotId: string) {
      this.loading = true;
      this.error = null;
      this.activeSnapshot = null;
      try {
        const detail = await backupShow(snapshotId);
        this.activeSnapshot = detail;
        return detail;
      } catch (err: any) {
        this.error = err?.message || 'Failed to fetch snapshot details';
        throw err;
      } finally {
        this.loading = false;
      }
    },

    async createBackup(repoId: string, options: ClientBackupOptions) {
      this.isBackingUp = true;
      this.backupProgress = 0;
      this.backupProgressMessage = 'Initializing backup...';
      const taskId = `backup-${repoId}-${Date.now()}`;
      this.activeBackupTaskId = taskId;
      let unlistenFn: (() => void) | null = null;
      try {
        unlistenFn = await listenProgress((event) => {
          if (event.taskId === taskId) {
            this.backupProgress = Math.round((event.current / (event.total || 1)) * 100);
            this.backupProgressMessage = event.message;
            if (event.done) {
              this.isBackingUp = false;
              this.activeBackupTaskId = null;
              if (unlistenFn) unlistenFn();
            }
          }
        });
        const snap = await backupCreate(repoId, options, taskId);
        await this.fetchSnapshots(repoId);
        return snap;
      } catch (err: any) {
        this.isBackingUp = false;
        this.activeBackupTaskId = null;
        if (unlistenFn) unlistenFn();
        throw err;
      }
    },

    async verifySnapshot(snapshotId: string) {
      this.isVerifying = true;
      this.verifyProgress = 0;
      this.verifyProgressMessage = 'Starting verification...';
      this.verifyReport = null;
      const taskId = `verify-${snapshotId}-${Date.now()}`;
      this.activeVerifyTaskId = taskId;
      let unlistenFn: (() => void) | null = null;
      try {
        unlistenFn = await listenProgress((event) => {
          if (event.taskId === taskId) {
            this.verifyProgress = Math.round((event.current / (event.total || 1)) * 100);
            this.verifyProgressMessage = event.message;
            if (event.done) {
              this.isVerifying = false;
              this.activeVerifyTaskId = null;
              if (unlistenFn) unlistenFn();
            }
          }
        });
        const report = await backupVerify(snapshotId, taskId);
        this.verifyReport = report;
        return report;
      } catch (err: any) {
        this.isVerifying = false;
        this.activeVerifyTaskId = null;
        if (unlistenFn) unlistenFn();
        throw err;
      }
    },

    async pruneBackups(repoId: string, keep: number) {
      this.isPruning = true;
      this.error = null;
      try {
        const report = await backupPrune(repoId, keep);
        this.pruneReport = report;
        await this.fetchSnapshots(repoId);
        return report;
      } catch (err: any) {
        this.error = err?.message || 'Prune failed';
        throw err;
      } finally {
        this.isPruning = false;
      }
    },

    async restoreRef(snapshotId: string, spec: ClientRestoreSpec) {
      this.isRestoring = true;
      this.error = null;
      try {
        const result = await restore(snapshotId, spec);
        return result;
      } catch (err: any) {
        this.error = err?.message || 'Restore failed';
        throw err;
      } finally {
        this.isRestoring = false;
      }
    },

    async cancelVerifyTask() {
      if (this.activeVerifyTaskId) {
        try {
          await cancel(this.activeVerifyTaskId);
        } catch (err) {
          console.error('Failed to cancel verification:', err);
        } finally {
          this.isVerifying = false;
          this.activeVerifyTaskId = null;
        }
      }
    },

    async cancelBackupTask() {
      if (this.activeBackupTaskId) {
        try {
          await cancel(this.activeBackupTaskId);
        } catch (err) {
          console.error('Failed to cancel backup:', err);
        } finally {
          this.isBackingUp = false;
          this.activeBackupTaskId = null;
        }
      }
    }
  }
});
