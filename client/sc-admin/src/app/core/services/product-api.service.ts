import { HttpClient, HttpParams } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable, from, switchMap } from 'rxjs';
import { ApiResponse, PaginatedResponse } from '../models/api-response.model';
import {
  AdminProductDTO,
  AvailableSizesResponse,
  CreateProductRequest,
  ProductFilterOptions,
  ProductListQuery,
  ProductPreviewResponse,
  UpdateProductRequest,
} from '../models/product.model';
import { ImageCompressionService } from './image-compression.service';

@Injectable({ providedIn: 'root' })
export class ProductApiService {
  private http = inject(HttpClient);
  private compression = inject(ImageCompressionService);
  private baseUrl = '/api/admin/products';

  getAvailableSizes(
    categoryId?: number,
    sizeGroup?: string,
    brandId?: number,
    gender?: string,
    status?: string,
    condition?: string,
    color?: string,
    sizeSystems?: string,
    priceMin?: string,
    priceMax?: string,
  ): Observable<ApiResponse<AvailableSizesResponse>> {
    let params = new HttpParams();
    if (categoryId) params = params.set('category_id', String(categoryId));
    if (sizeGroup) params = params.set('size_group', sizeGroup);
    if (brandId) params = params.set('brand_id', String(brandId));
    if (gender) params = params.set('gender', gender);
    if (status) params = params.set('status', status);
    if (condition) params = params.set('condition', condition);
    if (color) params = params.set('color', color);
    if (sizeSystems) params = params.set('size_systems', sizeSystems);
    if (priceMin) params = params.set('price_min', priceMin);
    if (priceMax) params = params.set('price_max', priceMax);
    return this.http.get<ApiResponse<AvailableSizesResponse>>(`${this.baseUrl}/available-sizes`, { params });
  }

  getFilterOptions(query?: Partial<ProductListQuery>): Observable<ApiResponse<ProductFilterOptions>> {
    let params = new HttpParams();
    if (query) {
      Object.entries(query).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          params = params.set(key, String(value));
        }
      });
    }
    return this.http.get<ApiResponse<ProductFilterOptions>>(`${this.baseUrl}/filter-options`, { params });
  }

  getProducts(query: ProductListQuery): Observable<ApiResponse<PaginatedResponse<ProductPreviewResponse>>> {
    let params = new HttpParams();
    Object.entries(query).forEach(([key, value]) => {
      if (value !== undefined && value !== null) {
        params = params.set(key, String(value));
      }
    });
    return this.http.get<ApiResponse<PaginatedResponse<ProductPreviewResponse>>>(this.baseUrl, { params });
  }

  getProduct(id: string): Observable<ApiResponse<AdminProductDTO>> {
    return this.http.get<ApiResponse<AdminProductDTO>>(`${this.baseUrl}/${id}`);
  }

  createProduct(request: CreateProductRequest, preview?: File, images?: File[]): Observable<ApiResponse<AdminProductDTO>> {
    return from(this.buildProductFormData(request, preview, images)).pipe(
      switchMap(formData => this.http.post<ApiResponse<AdminProductDTO>>(this.baseUrl, formData)),
    );
  }

  updateProduct(id: string, request: UpdateProductRequest, preview?: File, images?: File[]): Observable<ApiResponse<AdminProductDTO>> {
    return from(this.buildProductFormData(request, preview, images)).pipe(
      switchMap(formData => this.http.put<ApiResponse<AdminProductDTO>>(`${this.baseUrl}/${id}`, formData)),
    );
  }

  deleteProduct(id: string, expectedVersion: number): Observable<ApiResponse<null>> {
    const params = new HttpParams().set('expected_version', String(expectedVersion));
    return this.http.delete<ApiResponse<null>>(`${this.baseUrl}/${id}`, { params });
  }

  uploadImages(id: string, files: File[]): Observable<ApiResponse<{ queued: true; job_id: number }>> {
    return from(this.compression.compressMany(files)).pipe(
      switchMap(compressed => {
        const formData = new FormData();
        compressed.forEach(file => formData.append('images', file));
        return this.http.post<ApiResponse<{ queued: true; job_id: number }>>(
          `${this.baseUrl}/${id}/images`,
          formData,
        );
      }),
    );
  }

  uploadPreview(id: string, file: File): Observable<ApiResponse<{ queued: true; job_id: number }>> {
    return from(this.compression.compress(file)).pipe(
      switchMap(compressed => {
        const formData = new FormData();
        formData.append('preview', compressed);
        return this.http.post<ApiResponse<{ queued: true; job_id: number }>>(
          `${this.baseUrl}/${id}/preview`,
          formData,
        );
      }),
    );
  }

  private async buildProductFormData(
    request: CreateProductRequest | UpdateProductRequest,
    preview?: File,
    images?: File[],
  ): Promise<FormData> {
    const formData = new FormData();
    formData.append('metadata', JSON.stringify(request));
    if (preview) {
      formData.append('preview', await this.compression.compress(preview));
    }
    if (images?.length) {
      const compressed = await this.compression.compressMany(images);
      compressed.forEach(file => formData.append('images', file));
    }
    return formData;
  }
}
