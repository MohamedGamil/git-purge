import { describe, it, expect, beforeEach, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useToastStore } from './toast';

describe('useToastStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.useFakeTimers();
  });

  it('should initialize with an empty list of toasts', () => {
    const store = useToastStore();
    expect(store.toasts).toEqual([]);
  });

  it('should add a toast and automatically remove it after duration', () => {
    const store = useToastStore();
    const id = store.add('Test message', 'success', 3000);

    expect(store.toasts).toHaveLength(1);
    expect(store.toasts[0]).toEqual({
      id,
      message: 'Test message',
      type: 'success',
      duration: 3000
    });

    vi.advanceTimersByTime(3000);
    expect(store.toasts).toHaveLength(0);
  });

  it('should manually remove a toast by id', () => {
    const store = useToastStore();
    const id = store.add('Another message', 'info', 0); // 0 means no auto-remove

    expect(store.toasts).toHaveLength(1);
    store.remove(id);
    expect(store.toasts).toHaveLength(0);
  });

  it('should support helper methods for different toast types', () => {
    const store = useToastStore();

    store.success('Success message');
    store.error('Error message');
    store.info('Info message');
    store.warning('Warning message');

    expect(store.toasts).toHaveLength(4);
    expect(store.toasts[0].type).toBe('success');
    expect(store.toasts[1].type).toBe('error');
    expect(store.toasts[2].type).toBe('info');
    expect(store.toasts[3].type).toBe('warning');
  });
});
