import { ApplicationRef } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { TranslateService } from '@ngx-translate/core';
import { of } from 'rxjs';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { LanguageService } from './language.service';

class TranslateStub {
  addLangs = vi.fn();
  setDefaultLang = vi.fn();
  use = vi.fn().mockReturnValue(of('ok'));
}

describe('LanguageService', () => {
  let service: LanguageService;
  let translate: TranslateStub;
  let appRef: { tick: ReturnType<typeof vi.fn> };
  const storage = new Map<string, string>();
  const mockLocalStorage: Storage = {
    getItem: (key: string) => storage.get(key) ?? null,
    setItem: (key: string, value: string) => { storage.set(key, value); },
    removeItem: (key: string) => { storage.delete(key); },
    clear: () => { storage.clear(); },
    key: (index: number) => Array.from(storage.keys())[index] ?? null,
    get length() { return storage.size; },
  };

  beforeEach(() => {
    storage.clear();
    vi.stubGlobal('localStorage', mockLocalStorage);
    translate = new TranslateStub();
    appRef = { tick: vi.fn() };

    TestBed.configureTestingModule({
      providers: [
        LanguageService,
        { provide: TranslateService, useValue: translate },
        { provide: ApplicationRef, useValue: appRef },
      ],
    });
    service = TestBed.inject(LanguageService);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('initializes with saved language from localStorage', () => {
    localStorage.setItem('sc_admin_lang', 'ru');
    service.initialize();

    expect(translate.addLangs).toHaveBeenCalledWith(['en', 'ru']);
    expect(translate.setDefaultLang).toHaveBeenCalledWith('en');
    expect(translate.use).toHaveBeenCalledWith('ru');
    expect(service.currentLang()).toBe('ru');
  });

  it('falls back to default when storage is empty or invalid', () => {
    localStorage.setItem('sc_admin_lang', 'fr');
    service.initialize();
    expect(translate.use).toHaveBeenCalledWith('en');
    expect(service.currentLang()).toBe('en');
  });

  it('setLanguage persists and updates signal', () => {
    service.setLanguage('ru');
    expect(translate.use).toHaveBeenCalledWith('ru');
    expect(localStorage.getItem('sc_admin_lang')).toBe('ru');
    expect(service.currentLang()).toBe('ru');
    expect(appRef.tick).toHaveBeenCalled();
  });

  it('toggleLanguage flips en <-> ru', () => {
    service.currentLang.set('en');
    service.toggleLanguage();
    expect(translate.use).toHaveBeenCalledWith('ru');

    translate.use.mockClear();
    service.currentLang.set('ru');
    service.toggleLanguage();
    expect(translate.use).toHaveBeenCalledWith('en');
  });
});
