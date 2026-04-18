export type BrandTier = 'mass' | 'premium' | 'luxury';

export interface Brand {
  id: number;
  name: string;
  code: string;
  tier: BrandTier;
  country: string | null;
  created_at: string;
}

export interface BrandShortResponse {
  id: number;
  name: string;
  tier: string;
}

export interface CreateBrandRequest {
  name: string;
  code: string;
  tier: string;
  country?: string;
}

export interface UpdateBrandRequest {
  name?: string;
  code?: string;
  tier?: string;
  country?: string;
}
