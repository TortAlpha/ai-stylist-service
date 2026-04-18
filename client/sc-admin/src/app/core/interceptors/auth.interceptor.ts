import { HttpHandlerFn, HttpInterceptorFn, HttpRequest } from '@angular/common/http';
import { inject } from '@angular/core';
import { from, switchMap, catchError, throwError } from 'rxjs';
import { AuthStore } from '../../store/auth.store';

export const authInterceptor: HttpInterceptorFn = (req: HttpRequest<unknown>, next: HttpHandlerFn) => {
  const authStore = inject(AuthStore);
  const token = authStore.accessToken();

  if (token && !req.url.includes('/auth/')) {
    req = req.clone({ setHeaders: { Authorization: `Bearer ${token}` } });
  }

  return next(req).pipe(
    catchError(err => {
      if (err.status === 401 && !req.url.includes('/auth/')) {
        return from(authStore.refreshTokens()).pipe(
          switchMap(success => {
            if (success) {
              const newReq = req.clone({
                setHeaders: { Authorization: `Bearer ${authStore.accessToken()}` },
              });
              return next(newReq);
            }
            authStore.logout();
            return throwError(() => err);
          }),
          catchError(() => {
            authStore.logout();
            return throwError(() => err);
          }),
        );
      }
      return throwError(() => err);
    }),
  );
};
