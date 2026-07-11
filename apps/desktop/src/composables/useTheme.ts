import { ref, onMounted, onUnmounted } from 'vue';

export type ThemeMode = 'light' | 'dark' | 'system';

const theme = ref<ThemeMode>('system');

export function useTheme() {
  const setTheme = (mode: ThemeMode) => {
    theme.value = mode;
    localStorage.setItem('gitpurge-theme', mode);
    updateDOM();
  };

  const updateDOM = () => {
    const root = document.documentElement;
    const mode = theme.value;

    if (mode === 'system') {
      root.removeAttribute('data-theme');
    } else {
      root.setAttribute('data-theme', mode);
    }
  };

  const handleSystemChange = () => {
    if (theme.value === 'system') {
      updateDOM();
    }
  };

  let mediaQuery: MediaQueryList | null = null;

  onMounted(() => {
    const saved = localStorage.getItem('gitpurge-theme') as ThemeMode | null;
    if (saved) {
      theme.value = saved;
    }
    updateDOM();

    // Listen to OS theme changes
    mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    mediaQuery.addEventListener('change', handleSystemChange);
  });

  onUnmounted(() => {
    if (mediaQuery) {
      mediaQuery.removeEventListener('change', handleSystemChange);
    }
  });

  return {
    theme,
    setTheme,
  };
}
