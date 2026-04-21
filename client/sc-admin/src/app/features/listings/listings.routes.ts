import { Routes } from '@angular/router';

export default [
  { path: 'products/:id/listings', loadComponent: () => import('./product-listings/product-listings.component').then(m => m.ProductListingsComponent) },
  { path: 'listings', loadComponent: () => import('./my-listings/my-listings.component').then(m => m.MyListingsComponent) },
] satisfies Routes;
