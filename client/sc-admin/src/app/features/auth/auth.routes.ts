import { Routes } from '@angular/router';

export default [
  {
    path: '',
    loadComponent: () => import('./login/login.component').then(m => m.LoginComponent),
  },
] satisfies Routes;
