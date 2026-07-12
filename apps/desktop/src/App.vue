<template>
  <div class="app-layout">
    <!-- Sidebar Navigation -->
    <aside class="sidebar">
      <div class="logo">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path d="M12 2C6.48 2 2 6.48 2 12C2 17.52 6.48 22 12 22C17.52 22 22 17.52 22 12C22 6.48 17.52 2 12 2ZM12 20C7.58 20 4 16.42 4 12C4 7.58 7.58 4 12 4C16.42 4 20 7.58 20 12C20 16.42 16.42 20 12 20ZM16 11H13V8C13 7.45 12.55 7 12 7C11.45 7 11 7.45 11 8V11H8C7.45 11 7 11.45 7 12C7 12.55 7.45 13 8 13H11V16C11 16.55 11.45 17 12 17C12.55 17 13 16.55 13 16V13H16C16.55 13 17 12.55 17 12C17 11.45 16.55 11 16 11Z" fill="currentColor"/>
        </svg>
        <span>Git Purge</span>
      </div>

      <nav class="nav-links">
        <router-link to="/" class="nav-item" active-class="active">
          <LayoutDashboard class="lucide-icon icon" />
          <span>Dashboard</span>
        </router-link>

        <router-link to="/branches" class="nav-item" active-class="active">
          <GitBranch class="lucide-icon icon" />
          <span>Branches</span>
        </router-link>

        <router-link to="/cleanup" class="nav-item" active-class="active">
          <Sparkles class="lucide-icon icon" />
          <span>Cleanup</span>
        </router-link>

        <router-link to="/backups" class="nav-item" active-class="active">
          <Database class="lucide-icon icon" />
          <span>Backups</span>
        </router-link>

        <router-link to="/diff" class="nav-item" active-class="active">
          <GitCompare class="lucide-icon icon" />
          <span>Compare / Diff</span>
        </router-link>

        <router-link to="/history" class="nav-item" active-class="active">
          <History class="lucide-icon icon" />
          <span>History</span>
        </router-link>

        <router-link to="/auth" class="nav-item" active-class="active">
          <Key class="lucide-icon icon" />
          <span>Authentication</span>
        </router-link>

        <router-link to="/settings" class="nav-item" active-class="active">
          <Settings class="lucide-icon icon" />
          <span>Settings</span>
        </router-link>
      </nav>

      <div class="sidebar-footer">
        <div v-if="!isOnline" class="network-warning" title="Your device is offline or the VPN connection is lost. Git remote synchronization is temporarily disabled.">
          <span class="warning-dot"></span> Offline
        </div>
        <span>v0.2.0</span>
      </div>
    </aside>

    <!-- Main Content Area -->
    <main class="main-content">
      <router-view />
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { useTheme } from './composables/useTheme';
import { settingsGet } from './api/ipc';
import {
  LayoutDashboard,
  GitBranch,
  Sparkles,
  Database,
  GitCompare,
  History,
  Key,
  Settings
} from '@lucide/vue';

useTheme();

const isOnline = ref(navigator.onLine);

const updateOnlineStatus = () => {
  isOnline.value = navigator.onLine;
};

onMounted(async () => {
  window.addEventListener('online', updateOnlineStatus);
  window.addEventListener('offline', updateOnlineStatus);

  try {
    const appSettings = await settingsGet();
    if (appSettings.dateFormat) {
      localStorage.setItem('gitpurge-date-format', appSettings.dateFormat);
    }
  } catch (err) {
    console.error('Failed to load application settings:', err);
  }
});

onUnmounted(() => {
  window.removeEventListener('online', updateOnlineStatus);
  window.removeEventListener('offline', updateOnlineStatus);
});
</script>

<style>
@import './styles/global.css';

.app-layout {
  display: flex;
  width: 100vw;
  height: 100vh;
  background-color: var(--bg);
}

/* Sidebar Styling */
.sidebar {
  width: 240px;
  background-color: var(--surface-variant);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.logo {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-lg);
  font-weight: 700;
  font-size: 18px;
  color: var(--primary);
  border-bottom: 1px solid var(--border);
}

.nav-links {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: var(--spacing-md) var(--spacing-sm);
  flex-grow: 1;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-sm) var(--spacing-md);
  color: var(--on-surface);
  text-decoration: none;
  font-size: 14px;
  font-weight: 500;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}

.nav-item:hover {
  background-color: rgba(255, 255, 255, 0.05);
  color: var(--on-surface-strong);
}

:root[data-theme="light"] .nav-item:hover {
  background-color: rgba(0, 0, 0, 0.03);
}

.nav-item.active {
  background-color: var(--selection);
  color: var(--primary);
}

.icon {
  width: 20px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.sidebar-footer {
  padding: var(--spacing-md);
  font-size: 11px;
  color: var(--muted);
  border-top: 1px solid var(--border);
  text-align: center;
}

/* Main Content Styling */
.main-content {
  flex-grow: 1;
  height: 100%;
  overflow: hidden;
  background-color: var(--bg);
}

.network-warning {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  background-color: rgba(239, 68, 68, 0.15);
  color: #ef4444;
  padding: 6px;
  border-radius: var(--radius-sm, 4px);
  margin-bottom: 8px;
  font-weight: 600;
  font-size: 11px;
  border: 1px solid rgba(239, 68, 68, 0.2);
}

.warning-dot {
  width: 8px;
  height: 8px;
  background-color: #ef4444;
  border-radius: 50%;
  display: inline-block;
  animation: pulse 1.5s infinite;
}

@keyframes pulse {
  0% { transform: scale(0.9); opacity: 0.6; }
  50% { transform: scale(1.15); opacity: 1; }
  100% { transform: scale(0.9); opacity: 0.6; }
}
</style>
