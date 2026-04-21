import { ApplicationConfig, provideBrowserGlobalErrorListeners, APP_INITIALIZER, inject, importProvidersFrom } from '@angular/core';
import { provideRouter } from '@angular/router';
import { provideHttpClient, withInterceptors } from '@angular/common/http';
import { provideAnimationsAsync } from '@angular/platform-browser/animations/async';
import { providePrimeNG } from 'primeng/config';
import Aura from '@primeng/themes/aura';
import { ConfirmationService, MessageService } from 'primeng/api';
import { TranslateModule } from '@ngx-translate/core';
import { provideTranslateHttpLoader } from '@ngx-translate/http-loader';
import { routes } from './app.routes';
import { authInterceptor } from './core/interceptors/auth.interceptor';
import { AuthStore } from './store/auth.store';
import { LanguageService } from './core/services/language.service';
import { ThemeService } from './core/services/theme.service';

function initializeApp() {
  const authStore = inject(AuthStore);
  const languageService = inject(LanguageService);
  const themeService = inject(ThemeService);
  return async () => {
    languageService.initialize();
    themeService.initialize();
    await authStore.initialize();
  };
}

export const appConfig: ApplicationConfig = {
  providers: [
    provideBrowserGlobalErrorListeners(),
    provideRouter(routes),
    provideHttpClient(withInterceptors([authInterceptor])),
    provideAnimationsAsync(),
    providePrimeNG({
      theme: {
        preset: Aura,
        options: {
          darkModeSelector: '.app-dark',
        },
      },
    }),
    importProvidersFrom(
      TranslateModule.forRoot({
        defaultLanguage: 'en',
      })
    ),
    ...provideTranslateHttpLoader({ prefix: '/assets/i18n/', suffix: '.json' }),
    MessageService,
    ConfirmationService,
    { provide: APP_INITIALIZER, useFactory: initializeApp, multi: true },
  ],
};
