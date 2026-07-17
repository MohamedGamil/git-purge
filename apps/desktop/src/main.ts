import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
import router from './router';

const app = createApp(App);
const pinia = createPinia();

app.use(pinia);
app.use(router);

// Disable right click and development/reload hotkeys in production
if (import.meta.env.PROD) {
  document.addEventListener('contextmenu', (e) => e.preventDefault());
  document.addEventListener('keydown', (e) => {
    // F12, Ctrl+Shift+I/J/C, Cmd+Option+I/J/C, Ctrl+R, Cmd+R
    const isDevToolsShortcut = 
      e.key === 'F12' || 
      ((e.ctrlKey || e.metaKey) && e.shiftKey && (e.key === 'I' || e.key === 'i' || e.key === 'J' || e.key === 'j' || e.key === 'C' || e.key === 'c')) ||
      ((e.metaKey && e.altKey) && (e.key === 'I' || e.key === 'i' || e.key === 'J' || e.key === 'j'));
    
    const isReloadShortcut = 
      (e.ctrlKey && (e.key === 'r' || e.key === 'R')) || 
      (e.metaKey && (e.key === 'r' || e.key === 'R'));

    if (isDevToolsShortcut || isReloadShortcut) {
      e.preventDefault();
    }
  });
}

app.mount('#app');
