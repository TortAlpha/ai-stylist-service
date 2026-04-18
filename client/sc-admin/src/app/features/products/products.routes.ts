import { Routes } from '@angular/router';

export default [
  { path: 'products', loadComponent: () => import('./product-list/product-list.component').then(m => m.ProductListComponent) },
  { path: 'products/new', loadComponent: () => import('./product-form-page/product-form-page.component').then(m => m.ProductFormPageComponent) },
  { path: 'products/:id/edit', loadComponent: () => import('./product-form-page/product-form-page.component').then(m => m.ProductFormPageComponent) },
  { path: 'products/:id/history', loadComponent: () => import('./product-history/product-history.component').then(m => m.ProductHistoryComponent) },
  { path: 'products/:id/images', loadComponent: () => import('./product-images/product-images.component').then(m => m.ProductImagesComponent) },
  { path: 'products/:id/tags', loadComponent: () => import('./product-tags/product-tags.component').then(m => m.ProductTagsComponent) },
  { path: 'products/:id', loadComponent: () => import('./product-detail/product-detail.component').then(m => m.ProductDetailComponent) },
] satisfies Routes;
