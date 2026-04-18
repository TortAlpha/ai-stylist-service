import { TestBed } from '@angular/core/testing';
import { ThemeService } from './theme.service';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

describe('ThemeService', () => {
  let service: ThemeService;
  const storage = new Map<string, string>();
  const mockLocalStorage: Storage = {
    getItem: (key: string) => storage.get(key) ?? null,
    setItem: (key: string, value: string) => {
      storage.set(key, value);
    },
    removeItem: (key: string) => {
      storage.delete(key);
    },
    clear: () => {
      storage.clear();
    },
    key: (index: number) => Array.from(storage.keys())[index] ?? null,
    get length() {
      return storage.size;
    },
  };

  beforeEach(() => {
    storage.clear();
    vi.stubGlobal('localStorage', mockLocalStorage);
    TestBed.configureTestingModule({});
    service = TestBed.inject(ThemeService);
    document.documentElement.classList.remove('app-dark');
    document.body.classList.remove('app-dark');
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    document.documentElement.classList.remove('app-dark');
    document.body.classList.remove('app-dark');
    document.documentElement.style.removeProperty('color-scheme');
  });

  it('initializes dark theme from storage', () => {
    localStorage.setItem('sc_admin_theme', 'dark');

    service.initialize();

    expect(service.isDark()).toBe(true);
    expect(document.documentElement.classList.contains('app-dark')).toBe(true);
    expect(document.body.classList.contains('app-dark')).toBe(true);
    expect(document.documentElement.style.colorScheme).toBe('dark');
  });

  it('initializes light theme by default', () => {
    service.initialize();

    expect(service.isDark()).toBe(false);
    expect(document.documentElement.classList.contains('app-dark')).toBe(false);
    expect(document.body.classList.contains('app-dark')).toBe(false);
    expect(document.documentElement.style.colorScheme).toBe('light');
    expect(localStorage.getItem('sc_admin_theme')).toBe('light');
  });

  it('toggles and persists theme value', () => {
    service.initialize();

    service.toggle();
    expect(service.isDark()).toBe(true);
    expect(document.documentElement.style.colorScheme).toBe('dark');
    expect(localStorage.getItem('sc_admin_theme')).toBe('dark');

    service.toggle();
    expect(service.isDark()).toBe(false);
    expect(document.documentElement.style.colorScheme).toBe('light');
    expect(localStorage.getItem('sc_admin_theme')).toBe('light');
  });
});
