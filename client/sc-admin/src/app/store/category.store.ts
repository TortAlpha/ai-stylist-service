import { computed, inject } from '@angular/core';
import { patchState, signalStore, withComputed, withMethods, withState } from '@ngrx/signals';
import { firstValueFrom } from 'rxjs';
import { CategoryApiService } from '../core/services/category-api.service';
import { CategoryFullResponse, CreateCategoryRequest, UpdateCategoryRequest } from '../core/models/category.model';
import { TreeNode } from 'primeng/api';

interface CategoryState {
  categories: CategoryFullResponse[];
  loading: boolean;
  error: string | null;
}

const initialState: CategoryState = {
  categories: [],
  loading: false,
  error: null,
};

function buildCategoryTree(categories: CategoryFullResponse[]): TreeNode[] {
  const map = new Map<number, TreeNode>();
  const roots: TreeNode[] = [];

  for (const cat of categories) {
    map.set(cat.id, {
      key: String(cat.id),
      label: `${cat.name} (${cat.gender}, ${cat.product_type})`,
      data: cat,
      children: [],
      expanded: true,
    });
  }

  for (const cat of categories) {
    const node = map.get(cat.id)!;
    if (cat.parent_id && map.has(cat.parent_id)) {
      map.get(cat.parent_id)!.children!.push(node);
    } else {
      roots.push(node);
    }
  }

  return roots;
}

export const CategoryStore = signalStore(
  { providedIn: 'root' },
  withState(initialState),
  withComputed((state) => ({
    categoryTree: computed(() => buildCategoryTree(state.categories())),
  })),
  withMethods((store, categoryApi = inject(CategoryApiService)) => ({
    async loadCategories(): Promise<void> {
      patchState(store, { loading: true, error: null });
      try {
        const res = await firstValueFrom(categoryApi.getAll());
        if (res.success && res.data) {
          patchState(store, { categories: res.data, loading: false });
        } else {
          patchState(store, { loading: false, error: res.error ?? 'Failed to load categories' });
        }
      } catch (err: any) {
        patchState(store, { loading: false, error: err.error?.error ?? 'Failed to load categories' });
      }
    },

    async createCategory(request: CreateCategoryRequest): Promise<CategoryFullResponse | null> {
      patchState(store, { loading: true, error: null });
      try {
        const res = await firstValueFrom(categoryApi.create(request));
        if (res.success && res.data) {
          patchState(store, {
            categories: [...store.categories(), res.data],
            loading: false,
          });
          return res.data;
        }
        patchState(store, { loading: false, error: res.error ?? 'Failed to create category' });
        return null;
      } catch (err: any) {
        patchState(store, { loading: false, error: err.error?.error ?? 'Failed to create category' });
        return null;
      }
    },

    async updateCategory(id: number, request: UpdateCategoryRequest): Promise<CategoryFullResponse | null> {
      patchState(store, { loading: true, error: null });
      try {
        const res = await firstValueFrom(categoryApi.update(id, request));
        if (res.success && res.data) {
          patchState(store, {
            categories: store.categories().map(c => c.id === id ? res.data! : c),
            loading: false,
          });
          return res.data;
        }
        patchState(store, { loading: false, error: res.error ?? 'Failed to update category' });
        return null;
      } catch (err: any) {
        patchState(store, { loading: false, error: err.error?.error ?? 'Failed to update category' });
        return null;
      }
    },

    async deleteCategory(id: number): Promise<boolean> {
      patchState(store, { loading: true, error: null });
      try {
        await firstValueFrom(categoryApi.delete(id));
        patchState(store, {
          categories: store.categories().filter(c => c.id !== id),
          loading: false,
        });
        return true;
      } catch (err: any) {
        patchState(store, { loading: false, error: err.error?.error ?? 'Failed to delete category' });
        return false;
      }
    },
  })),
);
