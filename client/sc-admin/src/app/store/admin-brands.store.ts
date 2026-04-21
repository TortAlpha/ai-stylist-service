import { inject } from '@angular/core';
import { patchState, signalStore, withMethods, withState } from '@ngrx/signals';
import { firstValueFrom } from 'rxjs';
import { BrandApiService } from '../core/services/brand-api.service';
import { Brand, CreateBrandRequest, UpdateBrandRequest } from '../core/models/brand.model';

export interface AdminBrandFilters {
  search?: string;
}

interface AdminBrandsState {
  brands: Brand[];
  filters: AdminBrandFilters;
  loading: boolean;
  error: string | null;
  total: number;
  page: number;
  perPage: number;
  totalPages: number;
}

const initialState: AdminBrandsState = {
  brands: [],
  filters: {},
  loading: false,
  error: null,
  total: 0,
  page: 1,
  perPage: 20,
  totalPages: 0,
};

export const AdminBrandsStore = signalStore(
  { providedIn: 'root' },
  withState(initialState),
  withMethods((store, brandApi = inject(BrandApiService)) => ({
    async loadBrands(): Promise<void> {
      patchState(store, { loading: true, error: null });
      try {
        const { search } = store.filters();
        const res = await firstValueFrom(brandApi.search(search, store.page(), store.perPage()));
        if (res.success && res.data) {
          patchState(store, {
            brands: res.data.items,
            total: res.data.total,
            page: res.data.page,
            perPage: res.data.per_page,
            totalPages: res.data.total_pages,
            loading: false,
          });
        } else {
          patchState(store, { loading: false, error: res.error ?? 'Failed to load brands' });
        }
      } catch (err: any) {
        patchState(store, { loading: false, error: err.error?.error ?? 'Failed to load brands' });
      }
    },

    async setFilters(filters: AdminBrandFilters): Promise<void> {
      patchState(store, { filters, page: 1 });
      await this.loadBrands();
    },

    async setPage(page: number, perPage?: number): Promise<void> {
      patchState(store, {
        page,
        ...(perPage !== undefined ? { perPage } : {}),
      });
      await this.loadBrands();
    },

    async createBrand(request: CreateBrandRequest): Promise<boolean> {
      patchState(store, { loading: true, error: null });
      try {
        const res = await firstValueFrom(brandApi.create(request));
        if (res.success) {
          patchState(store, { loading: false });
          await this.loadBrands();
          return true;
        }
        patchState(store, { loading: false, error: res.error ?? 'Failed to create brand' });
        return false;
      } catch (err: any) {
        patchState(store, { loading: false, error: err.error?.error ?? 'Failed to create brand' });
        return false;
      }
    },

    async updateBrand(id: number, request: UpdateBrandRequest): Promise<boolean> {
      patchState(store, { loading: true, error: null });
      try {
        const res = await firstValueFrom(brandApi.update(id, request));
        if (res.success) {
          patchState(store, { loading: false });
          await this.loadBrands();
          return true;
        }
        patchState(store, { loading: false, error: res.error ?? 'Failed to update brand' });
        return false;
      } catch (err: any) {
        patchState(store, { loading: false, error: err.error?.error ?? 'Failed to update brand' });
        return false;
      }
    },

    async deleteBrand(id: number): Promise<boolean> {
      patchState(store, { loading: true, error: null });
      try {
        const res = await firstValueFrom(brandApi.delete(id));
        if (res.success) {
          patchState(store, {
            brands: store.brands().filter(b => b.id !== id),
            total: Math.max(0, store.total() - 1),
            loading: false,
          });
          return true;
        }
        patchState(store, { loading: false, error: res.error ?? 'Failed to delete brand' });
        return false;
      } catch (err: any) {
        patchState(store, { loading: false, error: err.error?.error ?? 'Failed to delete brand' });
        return false;
      }
    },
  })),
);
