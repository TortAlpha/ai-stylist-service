import { HttpClient } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';
import { ApiResponse } from '../models/api-response.model';
import { PurchaseLocation } from '../models/purchase-location.model';

@Injectable({ providedIn: 'root' })
export class PurchaseLocationApiService {
  private readonly http = inject(HttpClient);
  private readonly baseUrl = '/api/admin/purchase-locations';

  getAll(): Observable<ApiResponse<PurchaseLocation[]>> {
    return this.http.get<ApiResponse<PurchaseLocation[]>>(this.baseUrl);
  }
}

