import { inject } from '@angular/core';
import { patchState, signalStore, withMethods, withState } from '@ngrx/signals';
import { firstValueFrom, forkJoin } from 'rxjs';
import { TagApiService } from '../core/services/tag-api.service';
import { StyleTag, VibeTag, Season, CreateTagRequest } from '../core/models/tag.model';

interface TagState {
  styles: StyleTag[];
  vibes: VibeTag[];
  seasons: Season[];
  loading: boolean;
  error: string | null;
}

const initialState: TagState = {
  styles: [],
  vibes: [],
  seasons: [],
  loading: false,
  error: null,
};

export const TagStore = signalStore(
  { providedIn: 'root' },
  withState(initialState),
  withMethods((store, tagApi = inject(TagApiService)) => ({
    async loadAll(): Promise<void> {
      patchState(store, { loading: true, error: null });
      try {
        const [stylesRes, vibesRes, seasonsRes] = await firstValueFrom(
          forkJoin([tagApi.getStyles(), tagApi.getVibes(), tagApi.getSeasons()])
        );
        patchState(store, {
          styles: stylesRes.success && stylesRes.data ? stylesRes.data : [],
          vibes: vibesRes.success && vibesRes.data ? vibesRes.data : [],
          seasons: seasonsRes.success && seasonsRes.data ? seasonsRes.data : [],
          loading: false,
        });
      } catch (err: any) {
        patchState(store, { loading: false, error: err.error?.error ?? 'Failed to load tags' });
      }
    },

    async createStyle(request: CreateTagRequest): Promise<StyleTag | null> {
      patchState(store, { loading: true, error: null });
      try {
        const res = await firstValueFrom(tagApi.createStyle(request));
        if (res.success && res.data) {
          patchState(store, { styles: [...store.styles(), res.data], loading: false });
          return res.data;
        }
        patchState(store, { loading: false, error: res.error ?? 'Failed to create style tag' });
        return null;
      } catch (err: any) {
        patchState(store, { loading: false, error: err.error?.error ?? 'Failed to create style tag' });
        return null;
      }
    },

    async deleteStyle(id: number): Promise<boolean> {
      patchState(store, { loading: true, error: null });
      try {
        await firstValueFrom(tagApi.deleteStyle(id));
        patchState(store, { styles: store.styles().filter(s => s.id !== id), loading: false });
        return true;
      } catch (err: any) {
        patchState(store, { loading: false, error: err.error?.error ?? 'Failed to delete style tag' });
        return false;
      }
    },

    async createVibe(request: CreateTagRequest): Promise<VibeTag | null> {
      patchState(store, { loading: true, error: null });
      try {
        const res = await firstValueFrom(tagApi.createVibe(request));
        if (res.success && res.data) {
          patchState(store, { vibes: [...store.vibes(), res.data], loading: false });
          return res.data;
        }
        patchState(store, { loading: false, error: res.error ?? 'Failed to create vibe tag' });
        return null;
      } catch (err: any) {
        patchState(store, { loading: false, error: err.error?.error ?? 'Failed to create vibe tag' });
        return null;
      }
    },

    async deleteVibe(id: number): Promise<boolean> {
      patchState(store, { loading: true, error: null });
      try {
        await firstValueFrom(tagApi.deleteVibe(id));
        patchState(store, { vibes: store.vibes().filter(v => v.id !== id), loading: false });
        return true;
      } catch (err: any) {
        patchState(store, { loading: false, error: err.error?.error ?? 'Failed to delete vibe tag' });
        return false;
      }
    },

    async createSeason(request: CreateTagRequest): Promise<Season | null> {
      patchState(store, { loading: true, error: null });
      try {
        const res = await firstValueFrom(tagApi.createSeason(request));
        if (res.success && res.data) {
          patchState(store, { seasons: [...store.seasons(), res.data], loading: false });
          return res.data;
        }
        patchState(store, { loading: false, error: res.error ?? 'Failed to create season' });
        return null;
      } catch (err: any) {
        patchState(store, { loading: false, error: err.error?.error ?? 'Failed to create season' });
        return null;
      }
    },

    async deleteSeason(id: number): Promise<boolean> {
      patchState(store, { loading: true, error: null });
      try {
        await firstValueFrom(tagApi.deleteSeason(id));
        patchState(store, { seasons: store.seasons().filter(s => s.id !== id), loading: false });
        return true;
      } catch (err: any) {
        patchState(store, { loading: false, error: err.error?.error ?? 'Failed to delete season' });
        return false;
      }
    },
  })),
);
