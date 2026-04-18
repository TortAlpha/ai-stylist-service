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
  withMethods((store, authApi = inject(AuthApiService), userApi = inject(UserApiService), router = inject(Router)) => ({
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
      const refreshToken = store.refreshToken() ?? localStorage.getItem('refresh_token');
      if (!refreshToken) return false;
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
    },

    async loadUser(token?: string): Promise<void> {
      const accessToken = token ?? store.accessToken();
      if (!accessToken) return;
      try {
        const payload = JSON.parse(atob(accessToken.split('.')[1]));
        const userId = payload.sub;
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

    async initialize(): Promise<void> {
      const accessToken = localStorage.getItem('access_token');
      const refreshToken = localStorage.getItem('refresh_token');
      if (!refreshToken) return;

      patchState(store, { accessToken, refreshToken });

      const refreshed = await this.refreshTokens();
      if (refreshed) {
        await this.loadUser();
      } else {
        localStorage.removeItem('access_token');
        localStorage.removeItem('refresh_token');
        patchState(store, { ...initialState });
      }
    },
  })),
);
