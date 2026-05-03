import { inject } from '@angular/core';
import { patchState, signalStore, withMethods, withState } from '@ngrx/signals';
import { firstValueFrom } from 'rxjs';
import { ProductApiService } from '../core/services/product-api.service';
import {
  AvailableSizesResponse,
  ProductFilterOptions,
  ProductListQuery,
} from '../core/models/product.model';

export interface FilterOptionsScope {
  brand_id?: number;
  category_id?: number;
  gender?: string;
  status?: string;
  condition?: string;
  color?: string;
  price_min?: string;
  price_max?: string;
}

export interface AvailableSizesScope {
  category_id?: number;
  size_group?: string;
  brand_id?: number;
  gender?: string;
  status?: string;
  condition?: string;
  color?: string;
  size_systems?: string;
  price_min?: string;
  price_max?: string;
}

interface ProductFilterOptionsState {
  options: ProductFilterOptions | null;
  sizes: AvailableSizesResponse | null;
  loadingOptions: boolean;
  loadingSizes: boolean;
  error: string | null;
}

const initialState: ProductFilterOptionsState = {
  options: null,
  sizes: null,
  loadingOptions: false,
  loadingSizes: false,
  error: null,
};

export const ProductFilterOptionsStore = signalStore(
  { providedIn: 'root' },
  withState(initialState),
  withMethods((store, productApi = inject(ProductApiService)) => {
    let optionsRequestId = 0;
    let sizesRequestId = 0;

    return {
      async loadOptions(scope: FilterOptionsScope): Promise<void> {
        const requestId = ++optionsRequestId;
        patchState(store, { loadingOptions: true, error: null });
        try {
          const query: Partial<ProductListQuery> = {};
          if (scope.brand_id !== undefined) query.brand_id = scope.brand_id;
          if (scope.category_id !== undefined) query.category_id = scope.category_id;
          if (scope.gender) query.gender = scope.gender;
          if (scope.status) query.status = scope.status;
          if (scope.condition) query.condition = scope.condition;
          if (scope.color) query.color = scope.color;
          if (scope.price_min) query.price_min = scope.price_min;
          if (scope.price_max) query.price_max = scope.price_max;

          const res = await firstValueFrom(productApi.getFilterOptions(query));
          if (requestId !== optionsRequestId) return;
          if (res.success && res.data) {
            patchState(store, { options: res.data, loadingOptions: false });
          } else {
            patchState(store, {
              loadingOptions: false,
              error: res.error ?? 'Failed to load filter options',
            });
          }
        } catch (err: any) {
          if (requestId !== optionsRequestId) return;
          patchState(store, {
            loadingOptions: false,
            error: err?.error?.error ?? 'Failed to load filter options',
          });
        }
      },

      async loadSizes(scope: AvailableSizesScope): Promise<void> {
        if (!scope.category_id && !scope.size_group) {
          sizesRequestId += 1;
          patchState(store, { sizes: null, loadingSizes: false });
          return;
        }
        const requestId = ++sizesRequestId;
        patchState(store, { loadingSizes: true, error: null });
        try {
          const res = await firstValueFrom(
            productApi.getAvailableSizes(
              scope.category_id,
              scope.size_group,
              scope.brand_id,
              scope.gender,
              scope.status,
              scope.condition,
              scope.color,
              scope.size_systems,
              scope.price_min,
              scope.price_max,
            ),
          );
          if (requestId !== sizesRequestId) return;
          if (res.success && res.data) {
            patchState(store, { sizes: res.data, loadingSizes: false });
          } else {
            patchState(store, {
              sizes: null,
              loadingSizes: false,
              error: res.error ?? 'Failed to load sizes',
            });
          }
        } catch (err: any) {
          if (requestId !== sizesRequestId) return;
          patchState(store, {
            sizes: null,
            loadingSizes: false,
            error: err?.error?.error ?? 'Failed to load sizes',
          });
        }
      },

      clearSizes(): void {
        sizesRequestId += 1;
        patchState(store, { sizes: null, loadingSizes: false });
      },
    };
  }),
);
