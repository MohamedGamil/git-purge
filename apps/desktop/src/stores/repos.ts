import { defineStore } from 'pinia';
import {
  repoList,
  repoAdd,
  repoRemove,
  repoShow,
  scan,
  cancel,
  listenProgress,
  type RepoSummary,
  type RepoDetail,
  type Branch,
  type ClientScanOptions
} from '../api/ipc';

export const useReposStore = defineStore('repos', {
  state: () => ({
    repos: [] as RepoSummary[],
    activeRepoId: null as string | null,
    activeRepoDetail: null as RepoDetail | null,
    branches: [] as Branch[],
    scannedAt: null as string | null,
    loading: false,
    error: null as string | null,
    
    // Scan & Progress State
    isScanning: false,
    activeTaskId: null as string | null,
    scanProgress: 0,
    scanProgressMessage: '',
  }),

  actions: {
    async fetchRepos() {
      this.loading = true;
      this.error = null;
      try {
        const list = await repoList();
        this.repos = list.sort((a, b) => a.name.localeCompare(b.name));
      } catch (err: any) {
        this.error = err?.message || 'Failed to fetch repositories';
      } finally {
        this.loading = false;
      }
    },

    async addRepo(path?: string, url?: string, name?: string) {
      this.loading = true;
      this.error = null;
      try {
        const newRepo = await repoAdd(path, url, name);
        await this.fetchRepos();
        return newRepo;
      } catch (err: any) {
        this.error = err?.message || 'Failed to add repository';
        throw err;
      } finally {
        this.loading = false;
      }
    },

    async removeRepo(repoId: string, dropBackups?: boolean) {
      this.loading = true;
      this.error = null;
      try {
        await repoRemove(repoId, dropBackups);
        if (this.activeRepoId === repoId) {
          this.activeRepoId = null;
          this.activeRepoDetail = null;
          this.branches = [];
          this.scannedAt = null;
        }
        await this.fetchRepos();
      } catch (err: any) {
        this.error = err?.message || 'Failed to remove repository';
        throw err;
      } finally {
        this.loading = false;
      }
    },

    async selectRepo(repoId: string) {
      this.activeRepoId = repoId;
      this.loading = true;
      this.error = null;
      try {
        this.activeRepoDetail = await repoShow(repoId);
        // Try including remote tracking first, fall back to local-only if it fails (VPN/offline)
        try {
          await this.runScan(repoId, { includeRemote: true });
        } catch (scanErr: any) {
          console.warn('Remote check failed (possibly VPN offline). Falling back to local branch scan:', scanErr);
          await this.runScan(repoId, { includeRemote: false });
          this.error = 'Remote tracking unreachable (VPN/network offline). Displaying local branches only.';
        }
      } catch (err: any) {
        this.error = err?.message || 'Failed to load repository details';
      } finally {
        this.loading = false;
      }
    },

    async runScan(repoId: string, options: ClientScanOptions = {}) {
      this.isScanning = true;
      this.scanProgress = 0;
      this.scanProgressMessage = 'Initializing scan...';
      
      const taskId = `scan-${repoId}-${Date.now()}`;
      this.activeTaskId = taskId;

      let unlistenFn: (() => void) | null = null;

      try {
        // Set up the listener for progress updates
        unlistenFn = await listenProgress((event) => {
          if (event.taskId === taskId) {
            this.scanProgress = Math.round((event.current / (event.total || 1)) * 100);
            this.scanProgressMessage = event.message;
            if (event.done) {
              this.isScanning = false;
              this.activeTaskId = null;
              if (unlistenFn) unlistenFn();
            }
          }
        });

        const result = await scan(repoId, options, taskId);
        this.branches = result.branches;
        this.scannedAt = result.scannedAt;
        
        // Refresh the detail to show updated counts
        if (this.activeRepoId === repoId) {
          this.activeRepoDetail = await repoShow(repoId);
        }
        
        // Refresh repo list summaries
        await this.fetchRepos();

        this.isScanning = false;
        this.activeTaskId = null;
        if (unlistenFn) unlistenFn();
      } catch (err: any) {
        this.error = err?.message || 'Scan failed';
        this.isScanning = false;
        this.activeTaskId = null;
        if (unlistenFn) unlistenFn();
        throw err;
      }
    },

    async cancelActiveTask() {
      if (this.activeTaskId) {
        try {
          await cancel(this.activeTaskId);
        } catch (err) {
          console.error('Failed to cancel task:', err);
        } finally {
          this.isScanning = false;
          this.activeTaskId = null;
        }
      }
    }
  }
});
