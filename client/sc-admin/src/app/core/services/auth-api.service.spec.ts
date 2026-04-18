import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { AuthApiService } from './auth-api.service';

describe('AuthApiService', () => {
  let service: AuthApiService;
  let httpMock: HttpTestingController;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [
        AuthApiService,
        provideHttpClient(),
        provideHttpClientTesting(),
      ],
    });

    service = TestBed.inject(AuthApiService);
    httpMock = TestBed.inject(HttpTestingController);
  });

  afterEach(() => {
    httpMock.verify();
  });

  it('posts login credentials', () => {
    const body = { email: 'a@b.c', password: 'pw', device_type: 'web' };
    service.login(body).subscribe(res => {
      expect(res.access_token).toBe('at');
      expect(res.refresh_token).toBe('rt');
    });

    const req = httpMock.expectOne('/api/auth/login');
    expect(req.request.method).toBe('POST');
    expect(req.request.body).toEqual(body);
    req.flush({ access_token: 'at', refresh_token: 'rt' });
  });

  it('posts refresh token', () => {
    service.refresh({ refresh_token: 'rt' }).subscribe();

    const req = httpMock.expectOne('/api/auth/refresh');
    expect(req.request.method).toBe('POST');
    expect(req.request.body).toEqual({ refresh_token: 'rt' });
    req.flush({ access_token: 'at2', refresh_token: 'rt2' });
  });

  it('posts logout with refresh token in body', () => {
    service.logout('rt-logout').subscribe();

    const req = httpMock.expectOne('/api/auth/logout');
    expect(req.request.method).toBe('POST');
    expect(req.request.body).toEqual({ refresh_token: 'rt-logout' });
    req.flush(null);
  });
});
