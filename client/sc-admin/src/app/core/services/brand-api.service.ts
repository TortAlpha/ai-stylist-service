import { HttpClient, HttpParams } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';
import { ApiResponse, PaginatedResponse } from '../models/api-response.model';
import { Brand, CreateBrandRequest, UpdateBrandRequest } from '../models/brand.model';

@Injectable({ providedIn: 'root' })
export class BrandApiService {
  private http = inject(HttpClient);

  /** Public: returns flat array of all brands */
  getAll(): Observable<ApiResponse<Brand[]>> {
    return this.http.get<ApiResponse<Brand[]>>('/api/brands');
  }

  /** Admin: paginated search */
  search(searchTerm?: string, page?: number, perPage?: number): Observable<ApiResponse<PaginatedResponse<Brand>>> {
    let params = new HttpParams();
    if (searchTerm) params = params.set('search', searchTerm);
    if (page) params = params.set('page', String(page));
    if (perPage) params = params.set('per_page', String(perPage));
    return this.http.get<ApiResponse<PaginatedResponse<Brand>>>('/api/admin/brands', { params });
  }

  getById(id: number): Observable<ApiResponse<Brand>> {
    return this.http.get<ApiResponse<Brand>>(`/api/admin/brands/${id}`);
  }

  create(request: CreateBrandRequest): Observable<ApiResponse<Brand>> {
    return this.http.post<ApiResponse<Brand>>('/api/admin/brands', request);
  }

  update(id: number, request: UpdateBrandRequest): Observable<ApiResponse<Brand>> {
    return this.http.put<ApiResponse<Brand>>(`/api/admin/brands/${id}`, request);
  }

  delete(id: number): Observable<ApiResponse<null>> {
    return this.http.delete<ApiResponse<null>>(`/api/admin/brands/${id}`);
  }
}
