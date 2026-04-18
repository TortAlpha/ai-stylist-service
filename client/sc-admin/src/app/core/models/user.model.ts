export interface UserResponse {
  id: string;
  name: string;
  surname: string;
  email: string;
  phone_number: string | null;
  is_active: boolean;
  role: string;
  created_at: string;
  updated_at: string;
}

export interface CreateUserRequest {
  name: string;
  surname: string;
  email: string;
  password: string;
  phone_number?: string;
}

export interface UpdateUserRequest {
  name?: string;
  surname?: string;
  email?: string;
  phone_number?: string;
}

export interface AddressResponse {
  id: string;
  owner_id: string;
  street: string;
  building_num: string;
  floor_num: number | null;
  apartment_num: string | null;
  post_index: string;
  created_at: string;
  updated_at: string;
}

export interface CreateAddressRequest {
  street: string;
  building_num: string;
  floor_num?: number;
  apartment_num?: string;
  post_index: string;
}

export interface UpdateAddressRequest {
  street?: string;
  building_num?: string;
  floor_num?: number;
  apartment_num?: string;
  post_index?: string;
}
