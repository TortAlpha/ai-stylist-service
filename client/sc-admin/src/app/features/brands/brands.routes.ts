import { Routes } from '@angular/router';

export default [
  { path: 'brands', loadComponent: () => import('./brand-list/brand-list.component').then(m => m.BrandListComponent) },
  { path: 'brands/new', loadComponent: () => import('./brand-form-page/brand-form-page.component').then(m => m.BrandFormPageComponent) },
  { path: 'brands/:id/edit', loadComponent: () => import('./brand-form-page/brand-form-page.component').then(m => m.BrandFormPageComponent) },
] satisfies Routes;
