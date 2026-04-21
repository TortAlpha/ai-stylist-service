import { Routes } from '@angular/router';

export default [
  { path: 'profile', loadComponent: () => import('./my-profile/my-profile.component').then(m => m.MyProfileComponent) },
] satisfies Routes;
