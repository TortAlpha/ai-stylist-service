import { HttpClient } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';
import {
  AddressResponse,
  CreateAddressRequest,
  UpdateAddressRequest,
  UpdateUserRequest,
  UserResponse,
} from '../models/user.model';

@Injectable({ providedIn: 'root' })
export class UserApiService {
  private http = inject(HttpClient);
  private baseUrl = '/api/users';

  getUser(id: string): Observable<UserResponse> {
    return this.http.get<UserResponse>(`${this.baseUrl}/${id}`);
  }

  updateUser(id: string, request: UpdateUserRequest): Observable<UserResponse> {
    return this.http.patch<UserResponse>(`${this.baseUrl}/${id}`, request);
  }

  getAddresses(userId: string): Observable<AddressResponse[]> {
    return this.http.get<AddressResponse[]>(`${this.baseUrl}/${userId}/addresses`);
  }

  createAddress(userId: string, request: CreateAddressRequest): Observable<AddressResponse> {
    return this.http.post<AddressResponse>(`${this.baseUrl}/${userId}/addresses`, request);
  }

  updateAddress(userId: string, addressId: string, request: UpdateAddressRequest): Observable<AddressResponse> {
    return this.http.patch<AddressResponse>(`${this.baseUrl}/${userId}/addresses/${addressId}`, request);
  }

  deleteAddress(userId: string, addressId: string): Observable<void> {
    return this.http.delete<void>(`${this.baseUrl}/${userId}/addresses/${addressId}`);
  }
}
