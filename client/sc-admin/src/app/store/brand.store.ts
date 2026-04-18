import { inject } from '@angular/core';
import { patchState, signalStore, withMethods, withState } from '@ngrx/signals';
import { firstValueFrom } from 'rxjs';
import { BrandApiService } from '../core/services/brand-api.service';
import { Brand, CreateBrandRequest, UpdateBrandRequest } from '../core/models/brand.model';

interface BrandState {
  brands: Brand[];
  loading: boolean;
  error: string | null;
}

const initialState: BrandState = {
  brands: [],
  loading: false,
  error: null,
};

export const BrandStore = signalStore(
  { providedIn: 'root' },
  withState(initialState),
  withMethods((store, brandApi = inject(BrandApiService)) => ({
    async loadBrands(): Promise<void> {
      patchState(store, { loading: true, error: null });
      try {
        const res = await firstValueFrom(brandApi.getAll());
        if (res.success && res.data) {
          patchState(store, { brands: res.data, loading: false });
        } else {
          patchState(store, { loading: false, error: res.error ?? 'Failed to load brands' });
        }
      } catch (err: any) {
        patchState(store, { loading: false, error: err.error?.error ?? 'Failed to load brands' });
      }
    },

    async createBrand(request: CreateBrandRequest): Promise<Brand | null> {
      patchState(store, { loading: true, error: null });
      try {
        const res = await firstValueFrom(brandApi.create(request));
        if (res.success && res.data) {
          patchState(store, {
            brands: [...store.brands(), res.data],
            loading: false,
          });
          return res.data;
        }
        patchState(store, { loading: false, error: res.error ?? 'Failed to create brand' });
        return null;
      } catch (err: any) {
        patchState(store, { loading: false, error: err.error?.error ?? 'Failed to create brand' });
        return null;
      }
    },

    async updateBrand(id: number, request: UpdateBrandRequest): Promise<Brand | null> {
      patchState(store, { loading: true, error: null });
      try {
        const res = await firstValueFrom(brandApi.update(id, request));
        if (res.success && res.data) {
          patchState(store, {
            brands: store.brands().map(b => b.id === id ? res.data! : b),
            loading: false,
          });
          return res.data;
        }
        patchState(store, { loading: false, error: res.error ?? 'Failed to update brand' });
        return null;
      } catch (err: any) {
        patchState(store, { loading: false, error: err.error?.error ?? 'Failed to update brand' });
        return null;
      }
    },

    async deleteBrand(id: number): Promise<boolean> {
      patchState(store, { loading: true, error: null });
      try {
        await firstValueFrom(brandApi.delete(id));
        patchState(store, {
          brands: store.brands().filter(b => b.id !== id),
          loading: false,
        });
        return true;
      } catch (err: any) {
        patchState(store, { loading: false, error: err.error?.error ?? 'Failed to delete brand' });
        return false;
      }
    },
  })),
);
