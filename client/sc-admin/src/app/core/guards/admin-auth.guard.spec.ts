import { TestBed } from '@angular/core/testing';
import { Router, UrlTree, provideRouter } from '@angular/router';
import { adminAuthGuard } from './admin-auth.guard';
import { AuthStore } from '../../store/auth.store';

describe('adminAuthGuard', () => {
  function evaluateGuard(isLoggedIn: boolean, isAdmin: boolean) {
    TestBed.configureTestingModule({
      providers: [
        provideRouter([]),
        {
          provide: AuthStore,
          useValue: {
            isLoggedIn: () => isLoggedIn,
            isAdmin: () => isAdmin,
          },
        },
      ],
    });

    return TestBed.runInInjectionContext(() => adminAuthGuard({} as any, {} as any));
  }

  it('allows admin users', () => {
    const result = evaluateGuard(true, true);
    expect(result).toBe(true);
  });

  it('redirects logged-in non-admin users to login', () => {
    const result = evaluateGuard(true, false);
    const router = TestBed.inject(Router);

    expect(result instanceof UrlTree).toBe(true);
    expect(router.serializeUrl(result as UrlTree)).toBe('/login');
  });

  it('redirects anonymous users to login', () => {
    const result = evaluateGuard(false, false);
    const router = TestBed.inject(Router);

    expect(result instanceof UrlTree).toBe(true);
    expect(router.serializeUrl(result as UrlTree)).toBe('/login');
  });
});
