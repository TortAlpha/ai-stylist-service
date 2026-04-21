import { HttpClient, provideHttpClient, withInterceptors } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { describe, expect, it, beforeEach, vi } from 'vitest';
import { authInterceptor } from './auth.interceptor';
import { AuthStore } from '../../store/auth.store';

interface AuthStoreStub {
  accessToken: () => string | null;
  refreshTokens: ReturnType<typeof vi.fn>;
  logout: ReturnType<typeof vi.fn>;
}

function makeStore(initial: Partial<AuthStoreStub> = {}, tokens: string[] = []): AuthStoreStub {
  let idx = 0;
  return {
    accessToken: () => tokens[Math.min(idx, tokens.length - 1)] ?? null,
    refreshTokens: initial.refreshTokens ?? vi.fn().mockImplementation(() => {
      idx = Math.min(idx + 1, tokens.length - 1);
      return Promise.resolve(true);
    }),
    logout: initial.logout ?? vi.fn(),
  };
}

describe('authInterceptor', () => {
  let http: HttpClient;
  let httpMock: HttpTestingController;
  let store: AuthStoreStub;

  function configure(initialStore: AuthStoreStub) {
    TestBed.configureTestingModule({
      providers: [
        provideHttpClient(withInterceptors([authInterceptor])),
        provideHttpClientTesting(),
        { provide: AuthStore, useValue: initialStore },
      ],
    });
    http = TestBed.inject(HttpClient);
    httpMock = TestBed.inject(HttpTestingController);
  }

  it('attaches Authorization header when token exists and url is not /auth/', () => {
    store = makeStore({}, ['tok-1']);
    configure(store);

    http.get('/api/brands').subscribe();
    const req = httpMock.expectOne('/api/brands');
    expect(req.request.headers.get('Authorization')).toBe('Bearer tok-1');
    req.flush({});
    httpMock.verify();
  });

  it('does not attach header to /auth/ requests', () => {
    store = makeStore({}, ['tok-1']);
    configure(store);

    http.post('/api/auth/login', {}).subscribe();
    const req = httpMock.expectOne('/api/auth/login');
    expect(req.request.headers.has('Authorization')).toBe(false);
    req.flush({});
    httpMock.verify();
  });

  it('refreshes and retries on 401', async () => {
    store = makeStore({}, ['tok-old', 'tok-new']);
    configure(store);

    const result = new Promise<unknown>((resolve, reject) => {
      http.get('/api/brands').subscribe({ next: resolve, error: reject });
    });

    const first = httpMock.expectOne('/api/brands');
    expect(first.request.headers.get('Authorization')).toBe('Bearer tok-old');
    first.flush({}, { status: 401, statusText: 'Unauthorized' });

    await new Promise(r => setTimeout(r, 0));
    expect(store.refreshTokens).toHaveBeenCalledTimes(1);

    const retry = httpMock.expectOne('/api/brands');
    expect(retry.request.headers.get('Authorization')).toBe('Bearer tok-new');
    retry.flush({ ok: true });

    const body = await result;
    expect(body).toEqual({ ok: true });
    httpMock.verify();
  });

  it('logs out and rethrows when refresh fails', async () => {
    const refresh = vi.fn().mockResolvedValue(false);
    const logout = vi.fn();
    store = { accessToken: () => 'tok-old', refreshTokens: refresh, logout };
    configure(store);

    const error = new Promise<unknown>((_, reject) => {
      http.get('/api/brands').subscribe({ error: reject });
    });

    const first = httpMock.expectOne('/api/brands');
    first.flush({}, { status: 401, statusText: 'Unauthorized' });

    await expect(error).rejects.toBeDefined();
    expect(refresh).toHaveBeenCalled();
    expect(logout).toHaveBeenCalled();
    httpMock.verify();
  });

  it('does not try to refresh on 401 for /auth/ URL', async () => {
    const refresh = vi.fn();
    const logout = vi.fn();
    store = { accessToken: () => 'tok-x', refreshTokens: refresh, logout };
    configure(store);

    const error = new Promise<unknown>((_, reject) => {
      http.post('/api/auth/login', {}).subscribe({ error: reject });
    });

    const req = httpMock.expectOne('/api/auth/login');
    req.flush({}, { status: 401, statusText: 'Unauthorized' });

    await expect(error).rejects.toBeDefined();
    expect(refresh).not.toHaveBeenCalled();
    expect(logout).not.toHaveBeenCalled();
    httpMock.verify();
  });

  it('rethrows non-401 errors without refreshing', async () => {
    const refresh = vi.fn();
    const logout = vi.fn();
    store = { accessToken: () => 'tok-x', refreshTokens: refresh, logout };
    configure(store);

    const error = new Promise<unknown>((_, reject) => {
      http.get('/api/brands').subscribe({ error: reject });
    });

    const req = httpMock.expectOne('/api/brands');
    req.flush({}, { status: 500, statusText: 'Server Error' });

    await expect(error).rejects.toBeDefined();
    expect(refresh).not.toHaveBeenCalled();
    httpMock.verify();
  });
});
