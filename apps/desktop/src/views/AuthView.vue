<template>
  <div class="auth-container">
    <header class="view-header">
      <div>
        <h1>Remote Authentication</h1>
        <p class="subtitle">Securely manage credentials and SSH keys for remote branches and repository sync.</p>
      </div>
    </header>

    <div class="auth-layout">
      <!-- Unsupported banner indicating P6 phase -->
      <div v-if="unsupportedMsg" class="unsupported-banner card">
        <div class="banner-header">
          <span class="info-icon">🔑</span>
          <h3>Phase 6 Integration Preview</h3>
        </div>
        <p class="banner-body">{{ unsupportedMsg }}</p>
        <p class="banner-hint">Below is a visual layout preview of the remote credentials manager coming in the next phase.</p>
      </div>

      <!-- Preview layout (Premium feel, SAFE-07 compliant) -->
      <div class="auth-preview-dashboard">
        <div class="panel-grid">
          <!-- Left side: Credentials list -->
          <section class="card credentials-section">
            <h3>Configured Credentials</h3>
            <div class="credentials-list">
              <div class="credential-item card">
                <div class="cred-info">
                  <div class="cred-name">
                    <strong>github.com (MohamedGamil)</strong>
                    <span class="badge badge-success badge-tiny">Active</span>
                  </div>
                  <div class="cred-meta">Type: Personal Access Token (PAT) · Host: github.com</div>
                </div>
                <div class="cred-actions">
                  <button class="btn btn-secondary btn-sm" disabled>Test</button>
                  <button class="btn btn-danger-alt btn-sm" disabled>Remove</button>
                </div>
              </div>

              <div class="credential-item card">
                <div class="cred-info">
                  <div class="cred-name">
                    <strong>gitlab.company.com (mgamil-work)</strong>
                    <span class="badge badge-info badge-tiny">SSH Key</span>
                  </div>
                  <div class="cred-meta">Type: SSH Private Key (~/.ssh/id_ed25519) · Host: gitlab.company.com</div>
                </div>
                <div class="cred-actions">
                  <button class="btn btn-secondary btn-sm" disabled>Test</button>
                  <button class="btn btn-danger-alt btn-sm" disabled>Remove</button>
                </div>
              </div>
            </div>
          </section>

          <!-- Right side: Add credential form -->
          <section class="card add-cred-section">
            <h3>Add New Credential</h3>
            <form @submit.prevent class="add-cred-form">
              <div class="form-group">
                <label for="cred-host">Repository Host</label>
                <input id="cred-host" type="text" placeholder="e.g. github.com" class="form-input" disabled />
              </div>

              <div class="form-group">
                <label for="cred-type">Credential Type</label>
                <select id="cred-type" class="form-input" disabled>
                  <option>Personal Access Token (PAT)</option>
                  <option>SSH Private Key File</option>
                  <option>Username & Password</option>
                </select>
              </div>

              <div class="form-group">
                <label for="cred-username">Username</label>
                <input id="cred-username" type="text" placeholder="e.g. MohamedGamil" class="form-input" disabled />
              </div>

              <!-- SAFE-07: Secrets never rendered back or displayed plain-text in logs/inputs -->
              <div class="form-group">
                <label for="cred-token">Token / Private Key Content</label>
                <input id="cred-token" type="password" placeholder="••••••••••••••••••••••••" class="form-input" disabled />
              </div>

              <button class="btn btn-primary" disabled>💾 Save Credential</button>
            </form>
          </section>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { authList } from '../api/ipc';

const unsupportedMsg = ref<string | null>(null);

const checkAuthSupport = async () => {
  unsupportedMsg.value = null;
  try {
    await authList();
  } catch (err: any) {
    unsupportedMsg.value = err?.message || 'Authentication manager is unsupported in this version.';
  }
};

onMounted(() => {
  checkAuthSupport();
});
</script>

<style scoped>
.auth-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  padding: var(--spacing-lg);
  gap: var(--spacing-md);
  overflow-y: auto;
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

.auth-layout {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
}

.unsupported-banner {
  background-color: rgba(198, 120, 221, 0.05);
  border: 1px solid rgba(198, 120, 221, 0.2);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.banner-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  color: var(--accent-purple);
}

.banner-header h3 {
  font-size: 14px;
  font-weight: 600;
}

.banner-body {
  font-size: 13px;
  color: var(--on-surface);
}

.banner-hint {
  font-size: 11px;
  color: var(--muted);
}

.auth-preview-dashboard {
  opacity: 0.65;
  pointer-events: none;
}

.panel-grid {
  display: grid;
  grid-template-columns: 1.2fr 0.8fr;
  gap: var(--spacing-md);
}

@media (max-width: 768px) {
  .panel-grid {
    grid-template-columns: 1fr;
  }
}

.credentials-section h3, .add-cred-section h3 {
  font-size: 14px;
  color: var(--on-surface-strong);
  margin-bottom: var(--spacing-sm);
  border-bottom: 1px solid var(--border);
  padding-bottom: var(--spacing-xs);
}

.credentials-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.credential-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  background-color: var(--surface-raised);
}

.cred-name {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.cred-name strong {
  color: var(--on-surface-strong);
}

.cred-meta {
  font-size: 11px;
  color: var(--muted);
  margin-top: 2px;
}

.cred-actions {
  display: flex;
  gap: var(--spacing-xs);
}

.add-cred-form {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.form-group label {
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
  font-size: 13px;
  width: 100%;
}

.btn-danger-alt {
  background-color: transparent;
  color: var(--danger);
  border: 1px solid rgba(224, 108, 117, 0.3);
}

.badge-tiny {
  padding: 1px 4px;
  font-size: 9px;
}
</style>
