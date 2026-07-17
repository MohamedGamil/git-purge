import { defineStore } from 'pinia';
import {
  plan,
  deleteBranches,
  archiveBranches,
  cancel,
  listenProgress,
  diff,
  type ClientPlan,
  type ClientRunReport,
  type ClientActionFilter,
  type ClientExecOptions,
  type ClientDiffResult
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
  }),

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
            }
          }
        });
        const res = await deleteBranches(repoId, planData, execOpts, taskId);
        this.runReport = res;
        return res;
      } catch (err: any) {
        this.isExecuting = false;
        this.execTaskId = null;
        if (unlistenFn) unlistenFn();
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
            }
          }
        });
        const res = await archiveBranches(repoId, planData, execOpts, taskId);
        this.runReport = res;
        return res;
      } catch (err: any) {
        this.isExecuting = false;
        this.execTaskId = null;
        if (unlistenFn) unlistenFn();
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
