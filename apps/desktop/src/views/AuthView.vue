<template>
  <div class="auth-container">
    <header class="view-header">
      <div>
        <h1>Remote Authentication</h1>
        <p class="subtitle">Securely manage credentials and SSH keys for remote branch synchronizations.</p>
      </div>
    </header>

    <div class="auth-layout">
      <!-- Status Notice/Banner based on environment -->
      <div v-if="isKeyringUnsupported" class="unsupported-banner card">
        <div class="banner-header">
          <Key class="lucide-icon color-primary" style="margin-right: 6px;" />
          <h3>System Keyring Service Unavailable</h3>
        </div>
        <p class="banner-body">
          Native credentials keyring storage is not yet supported in this build. 
          Git operations will automatically fall back to using your default system SSH identities.
        </p>
        <p class="banner-hint">
          Currently running in <strong>Local Sandbox Mode</strong>. Credentials created here will be preserved in-memory for this UI session.
        </p>
      </div>

      <div v-else-if="isMockEnabled" class="mock-banner card">
        <div class="banner-header">
          <Info class="lucide-icon" style="margin-right: 6px;" />
          <h3>Mock Mode Enabled</h3>
        </div>
        <p class="banner-body">
          You are running the application in a standalone browser dev environment. 
          All authentication commands are simulated locally with high-fidelity mock data.
        </p>
      </div>

      <div class="panel-grid">
        <!-- Left side: Configured Credentials list -->
        <section class="card credentials-section">
          <h3>Configured Credentials</h3>
          
          <div v-if="loading" class="loading-state">
            <span class="spinner"></span>
            <p>Loading credentials...</p>
          </div>

          <div v-else class="credentials-list">
            <!-- Active Default System SSH Fallback Card -->
            <div class="credential-item system-fallback-card card">
              <div class="cred-info">
                <div class="cred-name">
                  <strong>Default SSH Identity (System Fallback)</strong>
                  <span class="badge badge-success badge-tiny">Active Fallback</span>
                </div>
                <div class="cred-meta">
                  Uses active <code>ssh-agent</code> or standard local SSH key files 
                  (<code>~/.ssh/id_ed25519</code>, <code>~/.ssh/id_rsa</code>, etc.)
                </div>
              </div>
              <div class="cred-actions">
                <span class="fallback-label">System Default</span>
              </div>
            </div>

            <!-- Stored credentials -->
            <div v-for="cred in credentials" :key="cred.id" class="credential-item card">
              <div class="cred-info">
                <div class="cred-name">
                  <strong>{{ cred.host }} ({{ cred.username }})</strong>
                  <span class="badge badge-tiny" :class="getBadgeClass(cred.kind)">
                    {{ getKindLabel(cred.kind) }}
                  </span>
                </div>
                <div class="cred-meta">
                  Provider: <code>{{ cred.provider }}</code> 
                  <span v-if="cred.meta.keyPath">· Path: <code>{{ cred.meta.keyPath }}</code></span>
                  <span v-if="cred.meta.tokenLast4">· Token: <code>••••{{ cred.meta.tokenLast4 }}</code></span>
                </div>
                
                <!-- Connection Test Result -->
                <div v-if="testResults[cred.id]" class="test-result-indicator" :class="{ 'success': testResults[cred.id].success }">
                  <span class="indicator-dot"></span>
                  {{ testResults[cred.id].message }}
                </div>
              </div>
              <div class="cred-actions">
                <button 
                  class="btn btn-secondary btn-sm" 
                  @click="testCredential(cred.id)" 
                  :disabled="testingId === cred.id"
                >
                  <span v-if="testingId === cred.id" class="spinner-tiny"></span>
                  <span v-else>Test</span>
                </button>
                <button 
                  class="btn btn-danger-alt btn-sm" 
                  @click="removeCredential(cred.id)"
                  :disabled="testingId === cred.id"
                >
                  Remove
                </button>
              </div>
            </div>
          </div>
        </section>

        <!-- Right side: Add credential form -->
        <section class="card add-cred-section">
          <h3>Add New Credential</h3>
          
          <form @submit.prevent="addCredential" class="add-cred-form">
            <div class="form-group">
              <label for="cred-host">Repository Host / Domain</label>
              <input 
                id="cred-host" 
                type="text" 
                v-model="formHost"
                placeholder="e.g. github.com, gitlab.acme.com" 
                class="form-input" 
                required
                :disabled="submitting"
              />
            </div>

            <div class="form-group">
              <label for="cred-type">Credential Type</label>
              <select id="cred-type" v-model="formKind" class="form-input" :disabled="submitting">
                <option value="ssh">SSH Private Key File</option>
                <option value="token">Personal Access Token (PAT)</option>
                <option value="basic">HTTPS Username & Password</option>
              </select>
            </div>

            <div class="form-group">
              <label for="cred-username">Username</label>
              <input 
                id="cred-username" 
                type="text" 
                v-model="formUsername"
                placeholder="e.g. git, oauth2, yourname" 
                class="form-input" 
                required
                :disabled="submitting"
              />
            </div>

            <!-- SSH Key Path -->
            <div class="form-group" v-if="formKind === 'ssh'">
              <label for="cred-keypath">Private Key Path</label>
              <input 
                id="cred-keypath" 
                type="text" 
                v-model="formKeyPath"
                placeholder="e.g. ~/.ssh/id_ed25519" 
                class="form-input"
                required
                :disabled="submitting"
              />
            </div>

            <!-- Token Field -->
            <div class="form-group" v-if="formKind === 'token'">
              <label for="cred-token">Personal Access Token</label>
              <input 
                id="cred-token" 
                type="password" 
                v-model="formToken"
                placeholder="Paste token contents..." 
                class="form-input"
                required
                :disabled="submitting"
              />
            </div>

            <!-- Password Field -->
            <div class="form-group" v-if="formKind === 'basic'">
              <label for="cred-password">Password / Secret</label>
              <input 
                id="cred-password" 
                type="password" 
                v-model="formPassword"
                placeholder="Enter password..." 
                class="form-input"
                required
                :disabled="submitting"
              />
            </div>

            <button 
              type="submit" 
              class="btn btn-primary w-100" 
              :disabled="submitting"
            >
              <span v-if="submitting">Saving Credential...</span>
              <span v-else><Save class="lucide-icon" style="margin-right: 4px;" /> Save Credential</span>
            </button>
          </form>
        </section>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { Key, Info, Save } from '@lucide/vue';
import { 
  authList, 
  authAdd, 
  authRemove, 
  authTest, 
  isMock 
} from '../api/ipc';

interface Credential {
  id: string;
  label: string;
  provider: string;
  host: string;
  username: string;
  kind: 'ssh' | 'token' | 'basic';
  meta: {
    keyPath?: string;
    tokenLast4?: string;
  };
}

const credentials = ref<Credential[]>([]);
const loading = ref(false);
const submitting = ref(false);
const testingId = ref<string | null>(null);

// Status markers
const isKeyringUnsupported = ref(false);
const isMockEnabled = ref(isMock);

// Form data
const formHost = ref('');
const formKind = ref<'ssh' | 'token' | 'basic'>('ssh');
const formUsername = ref('git');
const formKeyPath = ref('');
const formToken = ref('');
const formPassword = ref('');

// Test result mappings
const testResults = ref<Record<string, { success: boolean; message: string }>>({});

const loadCredentials = async () => {
  loading.value = true;
  isKeyringUnsupported.value = false;
  try {
    const rawList = await authList();
    credentials.value = rawList || [];
  } catch (err: any) {
    if (err?.code === 'UNSUPPORTED' || err?.message?.includes('UNSUPPORTED') || err?.message?.includes('not yet implemented')) {
      isKeyringUnsupported.value = true;
      // In local unsupported mode, let's load whatever mock credentials we have
      // so it degrades gracefully to sandbox fallback.
      try {
        // Fallback load mock list
        const fallbackList = await authList();
        credentials.value = fallbackList || [];
      } catch {
        credentials.value = [];
      }
    } else {
      console.error('Failed to load credentials:', err);
    }
  } finally {
    loading.value = false;
  }
};

const addCredential = async () => {
  submitting.value = true;
  const credId = 'auth-' + Math.random().toString(36).slice(2, 7);
  
  let label = '';
  let meta: any = {};
  
  if (formKind.value === 'ssh') {
    label = `SSH Key: ${formKeyPath.value || '~/.ssh/id_rsa'}`;
    meta = { keyPath: formKeyPath.value };
  } else if (formKind.value === 'token') {
    const last4 = formToken.value.slice(-4) || '••••';
    label = `Token (last 4: ${last4})`;
    meta = { tokenLast4: last4 };
  } else {
    label = `Basic Auth: ${formUsername.value}`;
  }

  const credential = {
    id: credId,
    label,
    provider: isKeyringUnsupported.value ? 'local-sandbox' : 'keyring',
    host: formHost.value,
    username: formUsername.value,
    kind: formKind.value,
    meta
  };

  try {
    await authAdd(credential);
    // Reset form fields
    formHost.value = '';
    formUsername.value = 'git';
    formKeyPath.value = '';
    formToken.value = '';
    formPassword.value = '';
    
    await loadCredentials();
  } catch (err: any) {
    alert('Failed to save credential: ' + (err?.message || err));
  } finally {
    submitting.value = false;
  }
};

const removeCredential = async (id: string) => {
  if (!confirm('Are you sure you want to remove this credential?')) return;
  try {
    await authRemove(id);
    if (testResults.value[id]) {
      delete testResults.value[id];
    }
    await loadCredentials();
  } catch (err: any) {
    alert('Failed to remove credential: ' + (err?.message || err));
  }
};

const testCredential = async (id: string) => {
  testingId.value = id;
  testResults.value[id] = null as any;
  try {
    const ok = await authTest(id);
    testResults.value[id] = {
      success: ok,
      message: ok ? 'Authentication check passed. Connection successful!' : 'Authentication failed.'
    };
  } catch (err: any) {
    testResults.value[id] = {
      success: false,
      message: err?.message || 'Connection test failed.'
    };
  } finally {
    testingId.value = null;
  }
};

const getBadgeClass = (kind: string) => {
  if (kind === 'ssh') return 'badge-info';
  if (kind === 'token') return 'badge-success';
  return 'badge-purple';
};

const getKindLabel = (kind: string) => {
  if (kind === 'ssh') return 'SSH Key';
  if (kind === 'token') return 'PAT Token';
  return 'Basic Auth';
};

onMounted(() => {
  loadCredentials();
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
  background-color: rgba(229, 192, 123, 0.05);
  border: 1px solid rgba(229, 192, 123, 0.2);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.mock-banner {
  background-color: rgba(97, 175, 239, 0.05);
  border: 1px solid rgba(97, 175, 239, 0.2);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
}

.banner-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  color: var(--warning);
}

.mock-banner .banner-header {
  color: var(--primary);
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

.loading-state {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  padding: var(--spacing-xl);
  color: var(--muted);
  gap: var(--spacing-md);
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
  border: 1px solid var(--border);
}

.system-fallback-card {
  border-left: 4px solid var(--success);
  background-color: rgba(152, 195, 121, 0.02);
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

.cred-meta code {
  font-family: var(--font-mono);
  background-color: var(--surface-variant);
  padding: 1px 4px;
  border-radius: 2px;
}

.cred-actions {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
}

.fallback-label {
  font-size: 11px;
  color: var(--muted);
  font-style: italic;
  padding-right: var(--spacing-xs);
}

.test-result-indicator {
  display: flex;
  align-items: center;
  gap: var(--spacing-xs);
  font-size: 11px;
  color: var(--danger);
  margin-top: var(--spacing-xs);
}

.test-result-indicator.success {
  color: var(--success);
}

.indicator-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background-color: var(--danger);
}

.success .indicator-dot {
  background-color: var(--success);
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

.form-input:focus {
  border-color: var(--primary);
}

.btn-danger-alt {
  background-color: transparent;
  color: var(--danger);
  border: 1px solid rgba(224, 108, 117, 0.3);
}

.btn-danger-alt:hover {
  background-color: rgba(224, 108, 117, 0.08);
}

.badge-tiny {
  padding: 1px 4px;
  font-size: 9px;
}

.spinner-tiny {
  display: inline-block;
  width: 10px;
  height: 10px;
  border: 2px solid var(--muted);
  border-top-color: var(--primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

.w-100 {
  width: 100%;
}
</style>
