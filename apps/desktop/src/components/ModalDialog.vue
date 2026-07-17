<template>
  <transition name="modal-fade">
    <div v-if="open" class="modal-overlay" @click.self="handleCancel" role="dialog" aria-modal="true">
      <div class="modal-box card" :class="['modal-' + type]" @keydown.esc="handleCancel">
        <!-- Header -->
        <div class="modal-header">
          <div class="modal-title-group">
            <TriangleAlert v-if="type === 'danger' || type === 'warning'" class="modal-type-icon text-warning-danger" />
            <Info v-else class="modal-type-icon text-info" />
            <h3>{{ title }}</h3>
          </div>
          <button class="modal-close-btn" @click="handleCancel" aria-label="Close dialog">
            <X class="lucide-icon" />
          </button>
        </div>

        <!-- Body -->
        <div class="modal-body">
          <p class="modal-message">{{ message }}</p>

          <!-- Type-to-confirm input -->
          <div v-if="requireConfirmationText" class="confirmation-input-group">
            <label for="modal-confirm-input" class="confirm-label">
              Please type <code class="confirm-token" title="Click to copy" @click="copyToken">{{ requireConfirmationText }}</code> to confirm:
            </label>
            <input
              id="modal-confirm-input"
              type="text"
              ref="inputRef"
              v-model="confirmInput"
              :placeholder="placeholder || 'Type matching text'"
              class="form-input confirm-field"
              @keydown.enter="handleConfirm"
              autocomplete="off"
            />
          </div>
        </div>

        <!-- Footer / Action Buttons -->
        <div class="modal-footer">
          <button type="button" class="btn btn-secondary" @click="handleCancel">
            {{ cancelText || 'Cancel' }}
          </button>
          <button
            type="button"
            class="btn"
            :class="[type === 'danger' ? 'btn-danger' : 'btn-primary']"
            :disabled="!isConfirmEnabled"
            @click="handleConfirm"
          >
            {{ confirmText || 'Confirm' }}
          </button>
        </div>
      </div>
    </div>
  </transition>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue';
import { TriangleAlert, Info, X } from '@lucide/vue';
import { useToastStore } from '../stores/toast';

const props = defineProps<{
  open: boolean;
  title: string;
  message: string;
  type?: 'info' | 'warning' | 'danger' | 'success';
  confirmText?: string;
  cancelText?: string;
  requireConfirmationText?: string;
  placeholder?: string;
}>();

const emit = defineEmits<{
  (e: 'update:open', val: boolean): void;
  (e: 'confirm'): void;
  (e: 'cancel'): void;
}>();

const toastStore = useToastStore();
const confirmInput = ref('');
const inputRef = ref<HTMLInputElement | null>(null);

const isConfirmEnabled = computed(() => {
  if (!props.requireConfirmationText) return true;
  return confirmInput.value.trim() === props.requireConfirmationText.trim();
});

const handleCancel = () => {
  emit('update:open', false);
  emit('cancel');
};

const handleConfirm = () => {
  if (!isConfirmEnabled.value) return;
  emit('update:open', false);
  emit('confirm');
};

const copyToken = async () => {
  if (!props.requireConfirmationText) return;
  try {
    await navigator.clipboard.writeText(props.requireConfirmationText);
    toastStore.success('Copied confirmation text to clipboard!');
  } catch (err: any) {
    toastStore.error('Failed to copy: ' + err.message);
  }
};

// Autofocus the input when open
watch(() => props.open, (newVal) => {
  if (newVal) {
    confirmInput.value = '';
    nextTick(() => {
      if (inputRef.value) {
        inputRef.value.focus();
      }
    });
  }
});
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: var(--overlay);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 9990;
  padding: var(--spacing-md);
}

.modal-box {
  width: 100%;
  max-width: 480px;
  background: rgba(25, 32, 42, 0.85);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border: 1px solid var(--border);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-lg);
  display: flex;
  flex-direction: column;
  padding: 0;
  overflow: hidden;
  animation: scaleUp 0.25s cubic-bezier(0.34, 1.56, 0.64, 1);
}

:root[data-theme="light"] .modal-box {
  background: rgba(255, 255, 255, 0.85);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--spacing-md) var(--spacing-lg);
  border-bottom: 1px solid var(--border);
}

.modal-title-group {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.modal-title-group h3 {
  font-size: 16px;
  color: var(--on-surface-strong);
  font-weight: 600;
}

.modal-type-icon {
  width: 20px;
  height: 20px;
  flex-shrink: 0;
}

.text-warning-danger {
  color: var(--danger);
}

.text-info {
  color: var(--info);
}

.modal-close-btn {
  background: none;
  border: none;
  color: var(--muted);
  cursor: pointer;
  padding: 4px;
  border-radius: var(--radius-xs);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: color var(--transition-fast), background-color var(--transition-fast);
}

.modal-close-btn:hover {
  color: var(--on-surface-strong);
  background-color: rgba(255, 255, 255, 0.1);
}

:root[data-theme="light"] .modal-close-btn:hover {
  background-color: rgba(0, 0, 0, 0.05);
}

.modal-body {
  padding: var(--spacing-lg);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
}

.modal-message {
  font-size: 14px;
  color: var(--on-surface);
  line-height: 1.6;
  white-space: pre-wrap;
}

.confirmation-input-group {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.confirm-label {
  font-size: 12px;
  color: var(--on-surface-strong);
  font-weight: 500;
}

.confirm-token {
  font-family: var(--font-mono);
  background: rgba(255, 255, 255, 0.1);
  padding: 2px 6px;
  border-radius: var(--radius-xs);
  color: var(--primary);
  font-weight: 600;
  cursor: pointer;
  border: 1px dashed rgba(255, 255, 255, 0.15);
  transition: all var(--transition-fast);
}

.confirm-token:hover {
  background: rgba(255, 255, 255, 0.2);
  border-color: var(--primary);
  transform: translateY(-0.5px);
}

:root[data-theme="light"] .confirm-token {
  background: rgba(0, 0, 0, 0.05);
  border: 1px dashed rgba(0, 0, 0, 0.1);
}

:root[data-theme="light"] .confirm-token:hover {
  background: rgba(0, 0, 0, 0.08);
  border-color: var(--primary);
}

.confirm-field {
  width: 100%;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--spacing-sm);
  padding: var(--spacing-md) var(--spacing-lg);
  background: rgba(0, 0, 0, 0.15);
  border-top: 1px solid var(--border);
}

:root[data-theme="light"] .modal-footer {
  background: rgba(0, 0, 0, 0.02);
}

/* Modal danger specific border style */
.modal-danger {
  border-top: 4px solid var(--danger);
}
.modal-warning {
  border-top: 4px solid var(--warning);
}

/* Transition Animations */
.modal-fade-enter-active,
.modal-fade-leave-active {
  transition: opacity var(--transition-normal);
}

.modal-fade-enter-from,
.modal-fade-leave-to {
  opacity: 0;
}

@keyframes scaleUp {
  from {
    transform: scale(0.95);
  }
  to {
    transform: scale(1);
  }
}
</style>
