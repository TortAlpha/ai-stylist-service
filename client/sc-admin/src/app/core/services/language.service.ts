import { ApplicationRef, Injectable, inject, signal } from '@angular/core';
import { TranslateService } from '@ngx-translate/core';

export type SupportedLang = 'en' | 'ru';

const LANG_KEY = 'sc_admin_lang';
const DEFAULT_LANG: SupportedLang = 'en';

@Injectable({ providedIn: 'root' })
export class LanguageService {
  private translate = inject(TranslateService);
  private appRef = inject(ApplicationRef);

  currentLang = signal<SupportedLang>(DEFAULT_LANG);

  initialize(): void {
    const saved = localStorage.getItem(LANG_KEY) as SupportedLang;
    const lang: SupportedLang = (saved === 'en' || saved === 'ru') ? saved : DEFAULT_LANG;
    this.translate.addLangs(['en', 'ru']);
    this.translate.setDefaultLang(DEFAULT_LANG);
    this.translate.use(lang);
    this.currentLang.set(lang);
  }

  setLanguage(lang: SupportedLang): void {
    this.translate.use(lang).subscribe(() => {
      this.currentLang.set(lang);
      this.appRef.tick();
    });
    localStorage.setItem(LANG_KEY, lang);
  }

  toggleLanguage(): void {
    const next: SupportedLang = this.currentLang() === 'en' ? 'ru' : 'en';
    this.setLanguage(next);
  }
}
