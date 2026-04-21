import { HttpClient } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';
import { ApiResponse, PaginatedResponse } from '../models/api-response.model';
import {
  CreateListingRequest,
  ListingWithProduct,
  Marketplace,
  ProductListing,
  UpdateListingStatusRequest,
} from '../models/listing.model';

@Injectable({ providedIn: 'root' })
export class ListingApiService {
  private http = inject(HttpClient);
  private baseUrl = '/api/listings';

  getAll(): Observable<ApiResponse<PaginatedResponse<ListingWithProduct>>> {
    return this.http.get<ApiResponse<PaginatedResponse<ListingWithProduct>>>(this.baseUrl);
  }

  getByProduct(productId: string): Observable<ApiResponse<PaginatedResponse<ProductListing>>> {
    return this.http.get<ApiResponse<PaginatedResponse<ProductListing>>>(`${this.baseUrl}/product/${productId}`);
  }

  getById(id: number): Observable<ApiResponse<ProductListing>> {
    return this.http.get<ApiResponse<ProductListing>>(`${this.baseUrl}/${id}`);
  }

  create(request: CreateListingRequest): Observable<ApiResponse<ProductListing>> {
    return this.http.post<ApiResponse<ProductListing>>(this.baseUrl, request);
  }

  updateStatus(id: number, request: UpdateListingStatusRequest): Observable<ApiResponse<ProductListing>> {
    return this.http.patch<ApiResponse<ProductListing>>(`${this.baseUrl}/${id}/status`, request);
  }

  delete(id: number): Observable<void> {
    return this.http.delete<void>(`${this.baseUrl}/${id}`);
  }

  getMarketplaces(): Observable<ApiResponse<Marketplace[]>> {
    return this.http.get<ApiResponse<Marketplace[]>>(`${this.baseUrl}/marketplaces`);
  }
}
