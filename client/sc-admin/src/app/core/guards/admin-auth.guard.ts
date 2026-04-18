import { inject } from '@angular/core';
import { CanActivateFn, Router } from '@angular/router';
import { AuthStore } from '../../store/auth.store';

export const adminAuthGuard: CanActivateFn = () => {
  const authStore = inject(AuthStore);
  const router = inject(Router);

  if (authStore.isLoggedIn() && authStore.isAdmin()) {
    return true;
  }

  return router.createUrlTree(['/login']);
};
