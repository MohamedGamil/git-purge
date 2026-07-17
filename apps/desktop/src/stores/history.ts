import { defineStore } from 'pinia';
import {
  historyGet,
  historyRunsGet,
  reportGenerate
} from '../api/ipc';

export const useHistoryStore = defineStore('history', {
  state: () => ({
    historyData: [] as any[],
    runs: [] as any[],
    loading: false,
    error: null as string | null,
    reportContent: '',
    isGeneratingReport: false
  }),

  actions: {
    async fetchHistory(repoId: string) {
      this.loading = true;
      this.error = null;
      try {
        const data = await historyGet(repoId);
        this.historyData = data;
        return data;
      } catch (err: any) {
        this.error = err?.message || 'Failed to fetch history';
        throw err;
      } finally {
        this.loading = false;
      }
    },

    async fetchRuns(repoId: string, limit: number, offset: number) {
      this.loading = true;
      this.error = null;
      try {
        const data = await historyRunsGet(repoId, limit, offset);
        // If offset is 0, replace, otherwise append
        if (offset === 0) {
          this.runs = data;
        } else {
          this.runs = [...this.runs, ...data];
        }
        return data;
      } catch (err: any) {
        this.error = err?.message || 'Failed to fetch runs';
        throw err;
      } finally {
        this.loading = false;
      }
    },

    async generateReport(repoId: string, format: string, reportType?: string) {
      this.isGeneratingReport = true;
      this.error = null;
      try {
        const report = await reportGenerate(repoId, format, reportType);
        this.reportContent = report.content;
        return report;
      } catch (err: any) {
        this.error = err?.message || 'Failed to generate report';
        throw err;
      } finally {
        this.isGeneratingReport = false;
      }
    }
  }
});
