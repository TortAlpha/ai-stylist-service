import { Routes } from '@angular/router';

export default [
  { path: '', redirectTo: 'products', pathMatch: 'full' },
  { path: '', loadChildren: () => import('../products/products.routes') },
  { path: '', loadChildren: () => import('../listings/listings.routes') },
  { path: '', loadChildren: () => import('../brands/brands.routes') },
  { path: '', loadChildren: () => import('../profile/profile.routes') },
] satisfies Routes;
