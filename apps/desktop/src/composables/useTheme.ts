import { ref, onMounted, onUnmounted, getCurrentInstance } from 'vue';

export type ThemeMode = 'light' | 'dark' | 'system';

const getSavedTheme = (): ThemeMode => {
  if (typeof localStorage !== 'undefined') {
    const saved = localStorage.getItem('gitpurge-theme');
    if (saved === 'light' || saved === 'dark' || saved === 'system') {
      return saved;
    }
  }
  return 'system';
};

const theme = ref<ThemeMode>(getSavedTheme());

export function useTheme() {
  // Re-evaluate theme value from localStorage when the composable is used
  theme.value = getSavedTheme();

  const setTheme = (mode: ThemeMode) => {
    theme.value = mode;
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('gitpurge-theme', mode);
    }
    updateDOM();
  };

  const updateDOM = () => {
    if (typeof document === 'undefined') return;
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

  // Run DOM update once immediately on execution
  updateDOM();

  if (getCurrentInstance()) {
    onMounted(() => {
      // Refresh value in case it changed elsewhere
      theme.value = getSavedTheme();
      updateDOM();

      // Listen to OS theme changes
      if (typeof window !== 'undefined' && window.matchMedia) {
        mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
        mediaQuery.addEventListener('change', handleSystemChange);
      }
    });

    onUnmounted(() => {
      if (mediaQuery) {
        mediaQuery.removeEventListener('change', handleSystemChange);
      }
    });
  }

  return {
    theme,
    setTheme,
  };
}
