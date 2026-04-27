import { computed, inject } from '@angular/core';
import { Router } from '@angular/router';
import { patchState, signalStore, withComputed, withMethods, withState } from '@ngrx/signals';
import { firstValueFrom } from 'rxjs';
import { AuthApiService } from '../core/services/auth-api.service';
import { UserApiService } from '../core/services/user-api.service';
import { UserResponse } from '../core/models/user.model';

interface AuthState {
  accessToken: string | null;
  refreshToken: string | null;
  user: UserResponse | null;
  loading: boolean;
  error: string | null;
}

const initialState: AuthState = {
  accessToken: null,
  refreshToken: null,
  user: null,
  loading: false,
  error: null,
};

export const AuthStore = signalStore(
  { providedIn: 'root' },
  withState(initialState),
  withComputed((state) => ({
    isLoggedIn: computed(() => !!state.accessToken()),
    isAdmin: computed(() => state.user()?.role === 'admin'),
    userId: computed(() => state.user()?.id ?? null),
    userName: computed(() => {
      const user = state.user();
      return user ? `${user.name} ${user.surname}` : null;
    }),
  })),
  withMethods((store, authApi = inject(AuthApiService), userApi = inject(UserApiService), router = inject(Router)) => {
    let inflightRefresh: Promise<boolean> | null = null;
    let crossTabBound = false;

    return ({
    async login(email: string, password: string): Promise<boolean> {
      patchState(store, { loading: true, error: null });
      try {
        const res = await firstValueFrom(authApi.login({ email, password }));
        const { access_token, refresh_token } = res;
        localStorage.setItem('access_token', access_token);
        localStorage.setItem('refresh_token', refresh_token);
        patchState(store, { accessToken: access_token, refreshToken: refresh_token });
        await this.loadUser(access_token);
        patchState(store, { loading: false });
        return true;
      } catch (err: any) {
        patchState(store, { loading: false, error: err.error?.error ?? 'Login failed' });
        return false;
      }
    },

    async refreshTokens(): Promise<boolean> {
      if (inflightRefresh) return inflightRefresh;
      const refreshToken = store.refreshToken() ?? localStorage.getItem('refresh_token');
      if (!refreshToken) return false;

      inflightRefresh = (async () => {
        try {
          const res = await firstValueFrom(authApi.refresh({ refresh_token: refreshToken }));
          const { access_token, refresh_token } = res;
          localStorage.setItem('access_token', access_token);
          localStorage.setItem('refresh_token', refresh_token);
          patchState(store, { accessToken: access_token, refreshToken: refresh_token });
          return true;
        } catch {
          return false;
        }
      })();

      try {
        return await inflightRefresh;
      } finally {
        inflightRefresh = null;
      }
    },

    setupCrossTabSync(): void {
      if (crossTabBound || typeof window === 'undefined') return;
      crossTabBound = true;
      window.addEventListener('storage', (event) => {
        if (event.storageArea !== localStorage) return;
        if (event.key !== 'access_token' && event.key !== 'refresh_token' && event.key !== null) return;

        const access = localStorage.getItem('access_token');
        const refresh = localStorage.getItem('refresh_token');

        if (!refresh) {
          if (store.refreshToken() === null && store.accessToken() === null) return;
          patchState(store, { ...initialState });
          if (!router.url.startsWith('/login')) router.navigate(['/login']);
          return;
        }

        if (refresh !== store.refreshToken() || access !== store.accessToken()) {
          patchState(store, { accessToken: access, refreshToken: refresh });
        }
      });
    },

    async loadUser(token?: string): Promise<void> {
      const accessToken = token ?? store.accessToken();
      if (!accessToken) return;
      try {
        const payloadSegment = accessToken.split('.')[1];
        if (!payloadSegment) return;

        const base64Payload = payloadSegment
          .replace(/-/g, '+')
          .replace(/_/g, '/')
          .padEnd(Math.ceil(payloadSegment.length / 4) * 4, '=');

        const payload = JSON.parse(atob(base64Payload));
        const userId = payload.sub;
        if (!userId) return;
        const user = await firstValueFrom(userApi.getUser(userId));
        patchState(store, { user });
      } catch {
        // Token decode or user fetch failed
      }
    },

    logout(): void {
      const refreshToken = store.refreshToken();
      if (refreshToken) {
        authApi.logout(refreshToken).subscribe();
      }
      localStorage.removeItem('access_token');
      localStorage.removeItem('refresh_token');
      patchState(store, { ...initialState });
      router.navigate(['/login']);
    },

    isAccessTokenExpired(token: string, leewaySeconds = 60): boolean {
      try {
        const payloadSegment = token.split('.')[1];
        if (!payloadSegment) return true;
        const base64Payload = payloadSegment
          .replace(/-/g, '+')
          .replace(/_/g, '/')
          .padEnd(Math.ceil(payloadSegment.length / 4) * 4, '=');
        const payload = JSON.parse(atob(base64Payload));
        if (typeof payload.exp !== 'number') return true;
        return payload.exp * 1000 - Date.now() < leewaySeconds * 1000;
      } catch {
        return true;
      }
    },

    async initialize(): Promise<void> {
      this.setupCrossTabSync();

      const accessToken = localStorage.getItem('access_token');
      const refreshToken = localStorage.getItem('refresh_token');
      if (!refreshToken) return;

      patchState(store, { accessToken, refreshToken });

      if (accessToken && !this.isAccessTokenExpired(accessToken)) {
        await this.loadUser(accessToken);
        return;
      }

      const refreshed = await this.refreshTokens();
      if (refreshed) {
        await this.loadUser();
      } else {
        localStorage.removeItem('access_token');
        localStorage.removeItem('refresh_token');
        patchState(store, { ...initialState });
      }
    },
    });
  }),
);
