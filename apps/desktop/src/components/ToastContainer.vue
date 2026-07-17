<template>
  <div class="toast-container">
    <transition-group name="toast-fade">
      <div
        v-for="toast in toastStore.toasts"
        :key="toast.id"
        class="toast-item"
        :class="['toast-' + toast.type]"
        role="alert"
      >
        <div class="toast-icon-wrapper">
          <CheckCircle v-if="toast.type === 'success'" class="lucide-icon text-success" />
          <AlertCircle v-else-if="toast.type === 'error'" class="lucide-icon text-danger" />
          <TriangleAlert v-else-if="toast.type === 'warning'" class="lucide-icon text-warning" />
          <Info v-else class="lucide-icon text-info" />
        </div>
        <div class="toast-message">{{ toast.message }}</div>
        <button class="toast-close-btn" @click="toastStore.remove(toast.id)" aria-label="Close notification">
          <X class="lucide-icon" />
        </button>
      </div>
    </transition-group>
  </div>
</template>

<script setup lang="ts">
import { useToastStore } from '../stores/toast';
import { CheckCircle, AlertCircle, TriangleAlert, Info, X } from '@lucide/vue';

const toastStore = useToastStore();
</script>

<style scoped>
.toast-container {
  position: fixed;
  bottom: var(--spacing-lg);
  right: var(--spacing-lg);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
  z-index: 9999;
  max-width: 400px;
  width: calc(100vw - 2 * var(--spacing-lg));
  pointer-events: none;
}

.toast-item {
  display: flex;
  align-items: flex-start;
  gap: var(--spacing-sm);
  padding: var(--spacing-sm) var(--spacing-md);
  border-radius: var(--radius-sm);
  background: rgba(25, 32, 42, 0.85);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  border: 1px solid var(--border);
  box-shadow: var(--shadow-md);
  pointer-events: auto;
  transition: all var(--transition-normal);
}

:root[data-theme="light"] .toast-item {
  background: rgba(255, 255, 255, 0.85);
}

/* Toast type colors & borders */
.toast-success {
  border-left: 4px solid var(--success);
}
.toast-error {
  border-left: 4px solid var(--danger);
}
.toast-warning {
  border-left: 4px solid var(--warning);
}
.toast-info {
  border-left: 4px solid var(--info);
}

.toast-icon-wrapper {
  flex-shrink: 0;
  margin-top: 2px;
}

.toast-message {
  flex-grow: 1;
  font-size: 13px;
  color: var(--on-surface-strong);
  font-weight: 500;
  word-break: break-word;
}

.toast-close-btn {
  background: none;
  border: none;
  color: var(--muted);
  cursor: pointer;
  padding: 2px;
  border-radius: var(--radius-xs);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  margin-top: 2px;
  transition: color var(--transition-fast), background-color var(--transition-fast);
}

.toast-close-btn:hover {
  color: var(--on-surface-strong);
  background-color: rgba(255, 255, 255, 0.1);
}

:root[data-theme="light"] .toast-close-btn:hover {
  background-color: rgba(0, 0, 0, 0.05);
}

.text-success { color: var(--success); }
.text-danger { color: var(--danger); }
.text-warning { color: var(--warning); }
.text-info { color: var(--info); }

/* Transitions */
.toast-fade-enter-from {
  opacity: 0;
  transform: translateY(20px) scale(0.95);
}
.toast-fade-leave-to {
  opacity: 0;
  transform: translateY(-20px) scale(0.95);
}
</style>
