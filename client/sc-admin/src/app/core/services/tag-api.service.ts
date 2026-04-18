import { HttpClient } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';
import { ApiResponse } from '../models/api-response.model';
import { CreateTagRequest, Season, StyleTag, VibeTag } from '../models/tag.model';

@Injectable({ providedIn: 'root' })
export class TagApiService {
  private http = inject(HttpClient);

  // Public read endpoints (return Vec<TagResponse>)
  getStyles(): Observable<ApiResponse<StyleTag[]>> {
    return this.http.get<ApiResponse<StyleTag[]>>('/api/tags/styles');
  }

  getVibes(): Observable<ApiResponse<VibeTag[]>> {
    return this.http.get<ApiResponse<VibeTag[]>>('/api/tags/vibes');
  }

  getSeasons(): Observable<ApiResponse<Season[]>> {
    return this.http.get<ApiResponse<Season[]>>('/api/tags/seasons');
  }

  // Admin write endpoints
  createStyle(request: CreateTagRequest): Observable<ApiResponse<StyleTag>> {
    return this.http.post<ApiResponse<StyleTag>>('/api/admin/tags/styles', request);
  }

  deleteStyle(id: number): Observable<void> {
    return this.http.delete<void>(`/api/admin/tags/styles/${id}`);
  }

  createVibe(request: CreateTagRequest): Observable<ApiResponse<VibeTag>> {
    return this.http.post<ApiResponse<VibeTag>>('/api/admin/tags/vibes', request);
  }

  deleteVibe(id: number): Observable<void> {
    return this.http.delete<void>(`/api/admin/tags/vibes/${id}`);
  }

  createSeason(request: CreateTagRequest): Observable<ApiResponse<Season>> {
    return this.http.post<ApiResponse<Season>>('/api/admin/tags/seasons', request);
  }

  deleteSeason(id: number): Observable<void> {
    return this.http.delete<void>(`/api/admin/tags/seasons/${id}`);
  }
}
