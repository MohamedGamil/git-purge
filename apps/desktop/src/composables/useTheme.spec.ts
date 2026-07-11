import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useTheme } from './useTheme';

describe('useTheme', () => {
  beforeEach(() => {
    localStorage.clear();
    document.documentElement.removeAttribute('data-theme');
    vi.restoreAllMocks();

    // Mock matchMedia
    Object.defineProperty(window, 'matchMedia', {
      writable: true,
      value: vi.fn().mockImplementation(query => ({
        matches: false,
        media: query,
        onchange: null,
        addListener: vi.fn(),
        removeListener: vi.fn(),
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
        dispatchEvent: vi.fn(),
      })),
    });
  });

  it('should initialize theme from localStorage or default to system', () => {
    const { theme } = useTheme();
    expect(theme.value).toBe('system');
    expect(document.documentElement.getAttribute('data-theme')).toBeNull();
  });

  it('should initialize theme from saved preference', () => {
    localStorage.setItem('gitpurge-theme', 'dark');
    const { theme } = useTheme();
    expect(theme.value).toBe('dark');
    expect(document.documentElement.getAttribute('data-theme')).toBe('dark');
  });

  it('should toggle theme and update DOM & localStorage', () => {
    const { theme, setTheme } = useTheme();
    
    setTheme('light');
    expect(theme.value).toBe('light');
    expect(localStorage.getItem('gitpurge-theme')).toBe('light');
    expect(document.documentElement.getAttribute('data-theme')).toBe('light');

    setTheme('dark');
    expect(theme.value).toBe('dark');
    expect(localStorage.getItem('gitpurge-theme')).toBe('dark');
    expect(document.documentElement.getAttribute('data-theme')).toBe('dark');

    setTheme('system');
    expect(theme.value).toBe('system');
    expect(localStorage.getItem('gitpurge-theme')).toBe('system');
    expect(document.documentElement.getAttribute('data-theme')).toBeNull();
  });
});
