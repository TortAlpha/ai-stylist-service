import { Component, ChangeDetectionStrategy, inject } from '@angular/core';
import { ButtonModule } from 'primeng/button';
import { ChipModule } from 'primeng/chip';
import { ToolbarModule } from 'primeng/toolbar';
import { TranslateModule } from '@ngx-translate/core';
import { AuthStore } from '../../../store/auth.store';
import { LanguageService } from '../../../core/services/language.service';
import { ThemeService } from '../../../core/services/theme.service';

@Component({
  selector: 'app-admin-header',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [ButtonModule, ChipModule, ToolbarModule, TranslateModule],
  templateUrl: './admin-header.component.html',
  styleUrl: './admin-header.component.scss',
})
export class AdminHeaderComponent {
  authStore = inject(AuthStore);
  langService = inject(LanguageService);
  themeService = inject(ThemeService);

  get userNameForChip(): string | undefined {
    return this.authStore.userName() ?? undefined;
  }

  get languageToggleLabel(): 'EN' | 'RU' {
    return this.langService.currentLang() === 'en' ? 'RU' : 'EN';
  }

  get themeToggleLabelKey(): string {
    return this.themeService.isDark() ? 'common.lightTheme' : 'common.darkTheme';
  }

  get themeToggleIcon(): string {
    return this.themeService.isDark() ? 'pi pi-sun' : 'pi pi-moon';
  }
}
