import { inject } from '@angular/core';
import { patchState, signalStore, withMethods, withState } from '@ngrx/signals';
import { firstValueFrom } from 'rxjs';
import { ProductApiService } from '../core/services/product-api.service';
import {
  ProductPreviewResponse,
  ProductListQuery,
  CreateProductRequest,
  UpdateProductRequest,
} from '../core/models/product.model';

export interface AdminProductFilters {
  search?: string;
  brand_id?: number;
  category_id?: number;
  status?: string;
  gender?: string;
  condition?: string;
  color?: string;
  price_min?: string;
  price_max?: string;
  size_value?: string;
  size_values?: string;
  size_values2?: string;
  size_system?: string;
  size_systems?: string;
  shoe_widths?: string;
  size_group?: string;
  sort_by?: 'price' | 'created_at' | 'updated_at' | 'name';
  sort_order?: 'asc' | 'desc';
}

interface AdminProductsState {
  products: ProductPreviewResponse[];
  filters: AdminProductFilters;
  loading: boolean;
  error: string | null;
  total: number;
  page: number;
  perPage: number;
  totalPages: number;
}

const initialState: AdminProductsState = {
  products: [],
  filters: {},
  loading: false,
  error: null,
  total: 0,
  page: 1,
  perPage: 20,
  totalPages: 0,
};

export const AdminProductsStore = signalStore(
  { providedIn: 'root' },
  withState(initialState),
  withMethods((store, productApi = inject(ProductApiService)) => ({
    async loadProducts(): Promise<void> {
      patchState(store, { loading: true, error: null });
      try {
        const filters = store.filters();
        const query: ProductListQuery = {
          page: store.page(),
          per_page: store.perPage(),
          ...filters,
        };
        const res = await firstValueFrom(productApi.getProducts(query));
        if (res.success && res.data) {
          patchState(store, {
            products: res.data.items,
            total: res.data.total,
            page: res.data.page,
            perPage: res.data.per_page,
            totalPages: res.data.total_pages,
            loading: false,
          });
        } else {
          patchState(store, { loading: false, error: res.error ?? 'Failed to load products' });
        }
      } catch (err: any) {
        patchState(store, {
          loading: false,
          error: err.error?.error ?? 'Failed to load products',
        });
      }
    },

    async setFilters(filters: AdminProductFilters): Promise<void> {
      patchState(store, { filters, page: 1 });
      await this.loadProducts();
    },

    async setPage(page: number, perPage?: number): Promise<void> {
      patchState(store, {
        page,
        ...(perPage !== undefined ? { perPage } : {}),
      });
      await this.loadProducts();
    },

    async createProduct(request: CreateProductRequest, preview?: File, images?: File[]): Promise<boolean> {
      patchState(store, { loading: true, error: null });
      try {
        const res = await firstValueFrom(productApi.createProduct(request, preview, images));
        if (res.success) {
          patchState(store, { loading: false });
          await this.loadProducts();
          return true;
        }
        patchState(store, { loading: false, error: res.error ?? 'Failed to create product' });
        return false;
      } catch (err: any) {
        patchState(store, {
          loading: false,
          error: err.error?.error ?? 'Failed to create product',
        });
        return false;
      }
    },

    async updateProduct(id: string, request: UpdateProductRequest, preview?: File, images?: File[]): Promise<{ success: boolean; staleVersion?: boolean }> {
      patchState(store, { loading: true, error: null });
      try {
        const res = await firstValueFrom(productApi.updateProduct(id, request, preview, images));
        if (!res.success) {
          const isStale = res.error?.toLowerCase().includes('version') || res.error?.toLowerCase().includes('conflict');
          patchState(store, { loading: false, error: res.error ?? 'Failed to update product' });
          return { success: false, staleVersion: isStale };
        }
        patchState(store, { loading: false });
        await this.loadProducts();
        return { success: true };
      } catch (err: any) {
        const errMsg: string = err.error?.error ?? err.message ?? 'Failed to update product';
        const isStale = err.status === 409;
        patchState(store, { loading: false, error: errMsg });
        return { success: false, staleVersion: isStale };
      }
    },

    async deleteProduct(id: string, expectedVersion: number): Promise<boolean> {
      patchState(store, { loading: true, error: null });
      try {
        const res = await firstValueFrom(productApi.deleteProduct(id, expectedVersion));
        if (res.success) {
          patchState(store, {
            products: store.products().filter(p => p.id !== id),
            total: store.total() - 1,
            loading: false,
          });
          return true;
        }
        patchState(store, { loading: false, error: res.error ?? 'Failed to delete product' });
        return false;
      } catch (err: any) {
        patchState(store, {
          loading: false,
          error: err.error?.error ?? 'Failed to delete product',
        });
        return false;
      }
    },
  }))
);
