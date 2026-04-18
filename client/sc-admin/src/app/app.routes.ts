import { Routes } from '@angular/router';
import { adminAuthGuard } from './core/guards/admin-auth.guard';

export const routes: Routes = [
  { path: '', pathMatch: 'full', redirectTo: 'admin/products' },
  {
    path: 'login',
    loadChildren: () => import('./features/auth/auth.routes'),
  },
  {
    path: 'admin',
    canActivate: [adminAuthGuard],
    loadComponent: () => import('./layouts/admin-layout/admin-layout.component').then(m => m.AdminLayoutComponent),
    loadChildren: () => import('./features/admin/admin.routes'),
  },
  { path: '**', redirectTo: 'admin/products' },
];
