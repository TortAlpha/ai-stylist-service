    export interface LoginRequest {
  email: string;
  password: string;
  device_type?: string;
}

export interface RefreshRequest {
  refresh_token: string;
}

export interface AuthResponse {
  access_token: string;
  refresh_token: string;
}
