import { computed, inject } from '@angular/core';
import { patchState, signalStore, withComputed, withMethods, withState } from '@ngrx/signals';
import { firstValueFrom } from 'rxjs';
import { UserApiService } from '../core/services/user-api.service';
import {
  AddressResponse,
  CreateAddressRequest,
  UpdateAddressRequest,
  UpdateUserRequest,
  UserResponse,
} from '../core/models/user.model';
import { AuthStore } from './auth.store';

interface UserProfileState {
  user: UserResponse | null;
  addresses: AddressResponse[];
  loading: boolean;
  addressesLoading: boolean;
  error: string | null;
}

const initialState: UserProfileState = {
  user: null,
  addresses: [],
  loading: false,
  addressesLoading: false,
  error: null,
};

export const UserProfileStore = signalStore(
  { providedIn: 'root' },
  withState(initialState),
  withComputed((state) => ({
    hasAddresses: computed(() => state.addresses().length > 0),
  })),
  withMethods(
    (
      store,
      userApi = inject(UserApiService),
      authStore = inject(AuthStore),
    ) => ({
      async loadUser(): Promise<void> {
        const userId = authStore.userId();
        if (!userId) return;

        patchState(store, { loading: true, error: null });
        try {
          const user = await firstValueFrom(userApi.getUser(userId));
          patchState(store, { user, loading: false });
        } catch (err: any) {
          patchState(store, { loading: false, error: err.error?.error ?? 'Failed to load profile' });
        }
      },

      async updateUser(request: UpdateUserRequest): Promise<boolean> {
        const userId = authStore.userId();
        if (!userId) return false;

        patchState(store, { loading: true, error: null });
        try {
          const user = await firstValueFrom(userApi.updateUser(userId, request));
          patchState(store, { user, loading: false });
          await authStore.loadUser();
          return true;
        } catch (err: any) {
          patchState(store, { loading: false, error: err.error?.error ?? 'Failed to update profile' });
          return false;
        }
      },

      async loadAddresses(): Promise<void> {
        const userId = authStore.userId();
        if (!userId) return;

        patchState(store, { addressesLoading: true, error: null });
        try {
          const addresses = await firstValueFrom(userApi.getAddresses(userId));
          patchState(store, { addresses, addressesLoading: false });
        } catch (err: any) {
          patchState(store, { addressesLoading: false, error: err.error?.error ?? 'Failed to load addresses' });
        }
      },

      async addAddress(request: CreateAddressRequest): Promise<boolean> {
        const userId = authStore.userId();
        if (!userId) return false;

        patchState(store, { addressesLoading: true, error: null });
        try {
          const address = await firstValueFrom(userApi.createAddress(userId, request));
          patchState(store, (state) => ({
            addresses: [...state.addresses, address],
            addressesLoading: false,
          }));
          return true;
        } catch (err: any) {
          patchState(store, { addressesLoading: false, error: err.error?.error ?? 'Failed to add address' });
          return false;
        }
      },

      async updateAddress(addressId: string, request: UpdateAddressRequest): Promise<boolean> {
        const userId = authStore.userId();
        if (!userId) return false;

        patchState(store, { addressesLoading: true, error: null });
        try {
          const updated = await firstValueFrom(userApi.updateAddress(userId, addressId, request));
          patchState(store, (state) => ({
            addresses: state.addresses.map((a) => (a.id === addressId ? updated : a)),
            addressesLoading: false,
          }));
          return true;
        } catch (err: any) {
          patchState(store, { addressesLoading: false, error: err.error?.error ?? 'Failed to update address' });
          return false;
        }
      },

      async deleteAddress(addressId: string): Promise<boolean> {
        const userId = authStore.userId();
        if (!userId) return false;

        patchState(store, { addressesLoading: true, error: null });
        try {
          await firstValueFrom(userApi.deleteAddress(userId, addressId));
          patchState(store, (state) => ({
            addresses: state.addresses.filter((a) => a.id !== addressId),
            addressesLoading: false,
          }));
          return true;
        } catch (err: any) {
          patchState(store, { addressesLoading: false, error: err.error?.error ?? 'Failed to delete address' });
          return false;
        }
      },
    }),
  ),
);
