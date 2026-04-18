import { HttpClient, HttpParams } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';
import { ApiResponse } from '../models/api-response.model';
import { CategoryFullResponse, CreateCategoryRequest, UpdateCategoryRequest } from '../models/category.model';

@Injectable({ providedIn: 'root' })
export class CategoryApiService {
  private http = inject(HttpClient);

  /** Public: returns flat array of categories */
  getAll(parentId?: number): Observable<ApiResponse<CategoryFullResponse[]>> {
    let params = new HttpParams();
    if (parentId !== undefined) params = params.set('parent_id', String(parentId));
    return this.http.get<ApiResponse<CategoryFullResponse[]>>('/api/categories', { params });
  }

  /** Admin: returns flat array of categories */
  getAllAdmin(parentId?: number): Observable<ApiResponse<CategoryFullResponse[]>> {
    let params = new HttpParams();
    if (parentId !== undefined) params = params.set('parent_id', String(parentId));
    return this.http.get<ApiResponse<CategoryFullResponse[]>>('/api/admin/categories', { params });
  }

  getById(id: number): Observable<ApiResponse<CategoryFullResponse>> {
    return this.http.get<ApiResponse<CategoryFullResponse>>(`/api/admin/categories/${id}`);
  }

  create(request: CreateCategoryRequest): Observable<ApiResponse<CategoryFullResponse>> {
    return this.http.post<ApiResponse<CategoryFullResponse>>('/api/admin/categories', request);
  }

  update(id: number, request: UpdateCategoryRequest): Observable<ApiResponse<CategoryFullResponse>> {
    return this.http.put<ApiResponse<CategoryFullResponse>>(`/api/admin/categories/${id}`, request);
  }

  delete(id: number): Observable<ApiResponse<null>> {
    return this.http.delete<ApiResponse<null>>(`/api/admin/categories/${id}`);
  }
}
