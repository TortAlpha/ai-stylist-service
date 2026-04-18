import { inject } from '@angular/core';
import { patchState, signalStore, withMethods, withState } from '@ngrx/signals';
import { firstValueFrom } from 'rxjs';
import { ListingApiService } from '../core/services/listing-api.service';
import {
  ProductListing,
  ListingWithProduct,
  Marketplace,
  CreateListingRequest,
  UpdateListingStatusRequest,
} from '../core/models/listing.model';

interface ListingState {
  listings: ListingWithProduct[];
  productListings: ProductListing[];
  marketplaces: Marketplace[];
  loading: boolean;
  error: string | null;
  total: number;
  page: number;
  perPage: number;
  totalPages: number;
}

const initialState: ListingState = {
  listings: [],
  productListings: [],
  marketplaces: [],
  loading: false,
  error: null,
  total: 0,
  page: 1,
  perPage: 12,
  totalPages: 0,
};

export const ListingStore = signalStore(
  { providedIn: 'root' },
  withState(initialState),
  withMethods((store, listingApi = inject(ListingApiService)) => ({
    async loadAllListings(): Promise<void> {
      patchState(store, { loading: true, error: null });
      try {
        const res = await firstValueFrom(listingApi.getAll());
        if (res.success && res.data) {
          patchState(store, { listings: res.data.items, loading: false });
        } else {
          patchState(store, { loading: false, error: res.error ?? 'Failed to load listings' });
        }
      } catch (err: any) {
        patchState(store, { loading: false, error: err.error?.error ?? 'Failed to load listings' });
      }
    },

    async loadProductListings(productId: string): Promise<void> {
      patchState(store, { loading: true, error: null });
      try {
        const res = await firstValueFrom(listingApi.getByProduct(productId));
        if (res.success && res.data) {
          patchState(store, { productListings: res.data.items, loading: false });
        } else {
          patchState(store, { loading: false, error: res.error ?? 'Failed to load listings' });
        }
      } catch (err: any) {
        patchState(store, { loading: false, error: err.error?.error ?? 'Failed to load listings' });
      }
    },

    async loadMarketplaces(): Promise<void> {
      try {
        const res = await firstValueFrom(listingApi.getMarketplaces());
        if (res.success && res.data) {
          patchState(store, { marketplaces: res.data });
        }
      } catch {
        // Silently fail — marketplaces are supplementary
      }
    },

    async createListing(request: CreateListingRequest): Promise<ProductListing | null> {
      patchState(store, { loading: true, error: null });
      try {
        const res = await firstValueFrom(listingApi.create(request));
        if (res.success && res.data) {
          patchState(store, {
            productListings: [...store.productListings(), res.data],
            loading: false,
          });
          return res.data;
        }
        patchState(store, { loading: false, error: res.error ?? 'Failed to create listing' });
        return null;
      } catch (err: any) {
        patchState(store, { loading: false, error: err.error?.error ?? 'Failed to create listing' });
        return null;
      }
    },

    async updateListing(id: number, request: UpdateListingStatusRequest): Promise<boolean> {
      patchState(store, { loading: true, error: null });
      try {
        const res = await firstValueFrom(listingApi.updateStatus(id, request));
        if (res.success && res.data) {
          const updated = res.data!;
          patchState(store, {
            productListings: store.productListings().map(l => l.id === id ? updated : l),
            listings: store.listings().map(l => l.id === id ? { ...l, ...updated } : l),
            loading: false,
          });
          return true;
        }
        patchState(store, { loading: false, error: res.error ?? 'Failed to update listing' });
        return false;
      } catch (err: any) {
        patchState(store, { loading: false, error: err.error?.error ?? 'Failed to update listing' });
        return false;
      }
    },

    async deleteListing(id: number): Promise<boolean> {
      patchState(store, { loading: true, error: null });
      try {
        await firstValueFrom(listingApi.delete(id));
        patchState(store, {
          productListings: store.productListings().filter(l => l.id !== id),
          listings: store.listings().filter(l => l.id !== id),
          loading: false,
        });
        return true;
      } catch (err: any) {
        patchState(store, { loading: false, error: err.error?.error ?? 'Failed to delete listing' });
        return false;
      }
    },
  }))
);
