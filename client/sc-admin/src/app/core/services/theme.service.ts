import { Injectable, signal } from '@angular/core';

const THEME_KEY = 'sc_admin_theme';

@Injectable({ providedIn: 'root' })
export class ThemeService {
  readonly isDark = signal(false);

  initialize(): void {
    const saved = localStorage.getItem(THEME_KEY);
    const dark = saved === 'dark';
    this.setDark(dark);
  }

  toggle(): void {
    this.setDark(!this.isDark());
  }

  private setDark(dark: boolean): void {
    this.isDark.set(dark);
    document.documentElement.classList.toggle('app-dark', dark);
    document.body.classList.toggle('app-dark', dark);
    document.documentElement.style.colorScheme = dark ? 'dark' : 'light';
    localStorage.setItem(THEME_KEY, dark ? 'dark' : 'light');
  }
}
