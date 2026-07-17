import { defineStore } from 'pinia';

export interface Toast {
  id: string;
  message: string;
  type: 'success' | 'error' | 'info' | 'warning';
  duration?: number;
}

export const useToastStore = defineStore('toast', {
  state: () => ({
    toasts: [] as Toast[]
  }),

  actions: {
    add(message: string, type: Toast['type'] = 'info', duration = 4000) {
      const id = Math.random().toString(36).substring(2, 9);
      const toast: Toast = { id, message, type, duration };
      this.toasts.push(toast);

      if (duration > 0) {
        setTimeout(() => {
          this.remove(id);
        }, duration);
      }
      return id;
    },

    remove(id: string) {
      this.toasts = this.toasts.filter(t => t.id !== id);
    },

    success(message: string, duration?: number) {
      return this.add(message, 'success', duration);
    },

    error(message: string, duration?: number) {
      return this.add(message, 'error', duration);
    },

    info(message: string, duration?: number) {
      return this.add(message, 'info', duration);
    },

    warning(message: string, duration?: number) {
      return this.add(message, 'warning', duration);
    }
  }
});
