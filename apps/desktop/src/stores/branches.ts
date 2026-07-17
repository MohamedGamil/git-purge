import { defineStore } from 'pinia';
import {
  plan,
  deleteBranches,
  archiveBranches,
  cancel,
  listenProgress,
  diff,
  getActiveCleanups,
  type ClientPlan,
  type ClientRunReport,
  type ClientActionFilter,
  type ClientExecOptions,
  type ClientDiffResult,
  type ActiveCleanupTask
} from '../api/ipc';

export const useBranchesStore = defineStore('branches', {
  state: () => ({
    planResult: null as ClientPlan | null,
    runReport: null as ClientRunReport | null,
    loadingPlan: false,
    planError: null as string | null,
    isExecuting: false,
    execProgress: 0,
    execProgressMessage: '',
    execTaskId: null as string | null,
    diffResult: null as ClientDiffResult | null,
    loadingDiff: false,
    activeCleanups: [] as ActiveCleanupTask[]
  }),

  getters: {
    runningCleanupsCount(state) {
      return state.activeCleanups.filter(c => c.status === 'running').length;
    }
  },

  actions: {
    async generatePlan(repoId: string, filter: ClientActionFilter) {
      this.loadingPlan = true;
      this.planError = null;
      this.planResult = null;
      try {
        const res = await plan(repoId, filter);
        this.planResult = res;
        return res;
      } catch (err: any) {
        this.planError = err?.message || 'Failed to generate plan';
        throw err;
      } finally {
        this.loadingPlan = false;
      }
    },

    async fetchActiveCleanups() {
      try {
        const tasks = (await getActiveCleanups()) || [];
        this.activeCleanups = tasks;
        
        // If we are not executing locally, but a task is running on the backend, reconnect to it
        const running = tasks.find(t => t.status === 'running');
        if (running && !this.isExecuting) {
          this.isExecuting = true;
          this.execTaskId = running.taskId;
          this.execProgress = Math.round((running.current / (running.total || 1)) * 100);
          this.execProgressMessage = running.message;
          
          let unlistenFn: (() => void) | null = null;
          unlistenFn = await listenProgress((event) => {
            if (event.taskId === running.taskId) {
              this.execProgress = Math.round((event.current / (event.total || 1)) * 100);
              this.execProgressMessage = event.message;
              if (event.done) {
                this.isExecuting = false;
                this.execTaskId = null;
                if (unlistenFn) unlistenFn();
                this.fetchActiveCleanups();
              }
            }
          });
        }
        return tasks;
      } catch (err) {
        console.error('Failed to fetch active cleanups:', err);
        throw err;
      }
    },

    async executeDelete(repoId: string, planData: ClientPlan, execOpts: ClientExecOptions) {
      this.isExecuting = true;
      this.execProgress = 0;
      this.execProgressMessage = 'Initializing branch deletion...';
      const taskId = `delete-${repoId}-${Date.now()}`;
      this.execTaskId = taskId;
      let unlistenFn: (() => void) | null = null;
      try {
        unlistenFn = await listenProgress((event) => {
          if (event.taskId === taskId) {
            this.execProgress = Math.round((event.current / (event.total || 1)) * 100);
            this.execProgressMessage = event.message;
            if (event.done) {
              this.isExecuting = false;
              this.execTaskId = null;
              if (unlistenFn) unlistenFn();
              this.fetchActiveCleanups();
            }
          }
        });
        // Add to activeCleanups immediately
        this.activeCleanups.push({
          taskId,
          repoId,
          kind: 'delete',
          status: 'running',
          current: 0,
          total: planData.actions.length,
          message: 'Initializing branch deletion...',
          startedAt: new Date().toISOString()
        });
        const res = await deleteBranches(repoId, planData, execOpts, taskId);
        this.runReport = res;
        this.fetchActiveCleanups();
        return res;
      } catch (err: any) {
        this.isExecuting = false;
        this.execTaskId = null;
        if (unlistenFn) unlistenFn();
        this.fetchActiveCleanups();
        throw err;
      }
    },

    async executeArchive(repoId: string, planData: ClientPlan, execOpts: ClientExecOptions) {
      this.isExecuting = true;
      this.execProgress = 0;
      this.execProgressMessage = 'Initializing branch archival...';
      const taskId = `archive-${repoId}-${Date.now()}`;
      this.execTaskId = taskId;
      let unlistenFn: (() => void) | null = null;
      try {
        unlistenFn = await listenProgress((event) => {
          if (event.taskId === taskId) {
            this.execProgress = Math.round((event.current / (event.total || 1)) * 100);
            this.execProgressMessage = event.message;
            if (event.done) {
              this.isExecuting = false;
              this.execTaskId = null;
              if (unlistenFn) unlistenFn();
              this.fetchActiveCleanups();
            }
          }
        });
        // Add to activeCleanups immediately
        this.activeCleanups.push({
          taskId,
          repoId,
          kind: 'archive',
          status: 'running',
          current: 0,
          total: planData.actions.length,
          message: 'Initializing branch archival...',
          startedAt: new Date().toISOString()
        });
        const res = await archiveBranches(repoId, planData, execOpts, taskId);
        this.runReport = res;
        this.fetchActiveCleanups();
        return res;
      } catch (err: any) {
        this.isExecuting = false;
        this.execTaskId = null;
        if (unlistenFn) unlistenFn();
        this.fetchActiveCleanups();
        throw err;
      }
    },

    async cancelActiveTask() {
      if (this.execTaskId) {
        try {
          await cancel(this.execTaskId);
        } catch (err) {
          console.error('Failed to cancel task:', err);
        } finally {
          this.isExecuting = false;
          this.execTaskId = null;
          this.fetchActiveCleanups();
        }
      }
    },

    async compareBranches(repoId: string, a: string, b: string) {
      this.loadingDiff = true;
      this.diffResult = null;
      try {
        const res = await diff(repoId, a, b);
        this.diffResult = res;
        return res;
      } catch (err: any) {
        throw err;
      } finally {
        this.loadingDiff = false;
      }
    },

    resetPlanAndReport() {
      this.planResult = null;
      this.runReport = null;
      this.planError = null;
      this.execProgress = 0;
      this.execProgressMessage = '';
      this.diffResult = null;
    }
  }
});
