<template>
  <div class="app-layout">
    <!-- Sidebar Navigation -->
    <aside class="sidebar">
      <router-link to="/" class="logo">
        <img :src="brandIcon" alt="Git Purge Logo" class="brand-logo" />
        <span class="app-name">Git Purge</span>
      </router-link>

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
        <div class="copyrights-n-version">
          <span>v0.3.3</span>
          <span>© {{ currentYear }} Git Purge</span>
          <span>By</span>
          <a class="author" href="https://github.com/MohamedGamil/git-purge" @click="handleOpenAuthorLink">M. Gamil</a>
        </div>
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
import brandIcon from './assets/brand_icon.png';
import { useTheme } from './composables/useTheme';
import { settingsGet, openUrl } from './api/ipc';
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

const currentYear = new Date().getFullYear();
const isOnline = ref(navigator.onLine);

const handleOpenAuthorLink = async (e: MouseEvent) => {
  e.preventDefault();
  try {
    await openUrl('https://github.com/MohamedGamil/git-purge');
  } catch (err) {
    console.error('Failed to open link:', err);
  }
};

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
  user-select: none;
  -webkit-user-select: none;
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
  text-decoration: none;
  cursor: pointer;
}

.brand-logo {
  width: 32px;
  height: 32px;
  object-fit: contain;
  border-radius: 6px;
  overflow: hidden;
}

.app-name {
  color: var(--on-surface-strong);
  font-size: 16px;
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

.copyrights-n-version {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  padding: 4px;
  margin-bottom: 4px;
  font-weight: 300;
  font-size: 9px;
  user-select: none;
  background-color: rgba(167, 156, 156, 0.05);
  border-radius: var(--radius-sm, 4px);
  color: var(--muted);
  font-family: var(--font-mono);
}

.author {
  color: var(--primary);
  text-decoration: none;
}

.author:hover {
  text-decoration: underline;
}

@keyframes pulse {
  0% { transform: scale(0.9); opacity: 0.6; }
  50% { transform: scale(1.15); opacity: 1; }
  100% { transform: scale(0.9); opacity: 0.6; }
}
</style>
