import { BrandShortResponse } from './brand.model';

export type ProductCondition = 'new_with_tags' | 'excellent' | 'good' | 'fair';
export type ProductStatus =
  | 'intake'
  | 'inspection'
  | 'rejected'
  | 'preparation'
  | 'photo_queue'
  | 'photo_done'
  | 'ready'
  | 'reserved'
  | 'sold'
  | 'returned';
export type ProductSortField = 'price' | 'created_at' | 'updated_at' | 'name';
export type SortOrder = 'asc' | 'desc';

export interface SizeResponse {
  size_group: string;
  size_value?: string;
  size_value2?: string;
  size_system?: string;
  measurement_cm?: string;
  size_label?: string;
}

export interface CategoryResponse {
  name: string;
  parent_category: string | null;
  gender: string;
  size_group: string;
}

export interface ImageVariantUrls {
  thumb: string;
  medium: string;
  full: string;
}

export interface ProductPreviewResponse {
  id: string;
  sku: string;
  name: string;
  purchase_price: string | null;
  currency: string;
  preview_url: ImageVariantUrls | null;
  brand_name: string;
  product_type: string;
  category: string;
  status?: string;
  condition?: string;
  color: string | null;
  size: SizeResponse;
}

export interface ProductAttributesResponse {
  material: string | null;
  condition: string;
  color: string | null;
  year_of_release: number | null;
  is_vintage: boolean;
  is_collab: boolean;
  collab_name: string | null;
  is_limited_edition: boolean;
  special_notes: string | null;
}

export type TypeDetailsResponse =
  | { type: 'clothing'; fit: string | null }
  | { type: 'footwear'; shoe_width: string | null; insole_length_cm: string | null }
  | { type: 'bags'; width_cm: string | null; height_cm: string | null; depth_cm: string | null; handle_type: string | null; bag_size_label: string | null }
  | { type: 'jewelry'; metal: string | null; stone: string | null; clasp_type: string | null }
  | { type: 'accessories' };

export interface ProductTagsResponse {
  styles: string[];
  vibes: string[];
  seasons: string[];
}

export interface AdminProductDTO {
  id: string;
  sku: string;
  name: string;
  purchase_price: string | null;
  purchase_location: string | null;
  currency: string;
  ai_notes: string | null;
  preview_url: ImageVariantUrls | null;
  image_urls: ImageVariantUrls[];
  brand: BrandShortResponse;
  brand_id: number;
  product_type: string;
  category: CategoryResponse;
  category_id: number;
  status: string;
  version: number;
  details: ProductAttributesResponse;
  size: SizeResponse;
  type_details: TypeDetailsResponse;
  tags: ProductTagsResponse;
  created_at: string;
  updated_at: string;
}

export interface SizeInput {
  size_value?: string;
  size_value2?: string;
  size_system?: string;
  measurement_cm?: string;
}

export interface CreateProductDetailsRequest {
  condition: ProductCondition;
  material?: string;
  color?: string;
  year_of_release?: number;
  is_vintage?: boolean;
  is_collab?: boolean;
  collab_name?: string;
  is_limited_edition?: boolean;
  special_notes?: string;
  size: SizeInput;
  type_details: TypeDetailsInput;
}

export interface UpdateProductDetailsRequest {
  condition?: ProductCondition;
  material?: string;
  color?: string;
  year_of_release?: number;
  is_vintage?: boolean;
  is_collab?: boolean;
  collab_name?: string;
  is_limited_edition?: boolean;
  special_notes?: string;
  size?: SizeInput;
  type_details?: TypeDetailsInput;
}

export type TypeDetailsInput =
  | { product_type: 'clothing'; fit?: string }
  | { product_type: 'footwear'; shoe_width?: string; insole_length_cm?: string }
  | { product_type: 'bags'; width_cm?: string; height_cm?: string; depth_cm?: string; handle_type?: string; bag_size_label?: string }
  | { product_type: 'jewelry'; metal?: string; stone?: string; clasp_type?: string }
  | { product_type: 'accessories' };

export interface CreateProductRequest {
  name: string;
  brand_id: number;
  category_id: number;
  status?: ProductStatus;
  purchase_price?: string;
  purchase_location_id?: number;
  currency?: string;
  ai_notes?: string;
  details: CreateProductDetailsRequest;
  style_tag_ids: number[];
  vibe_tag_ids: number[];
  season_ids: number[];
}

export interface UpdateProductRequest {
  name?: string;
  brand_id?: number;
  category_id?: number;
  status?: ProductStatus;
  purchase_price?: string;
  purchase_location_id?: number;
  currency?: string;
  ai_notes?: string;
  details?: UpdateProductDetailsRequest;
  style_tag_ids?: number[];
  vibe_tag_ids?: number[];
  season_ids?: number[];
  expected_version: number;
}

export interface ProductListQuery {
  page?: number;
  per_page?: number;
  search?: string;
  brand_id?: number;
  category_id?: number;
  product_type?: string;
  status?: string;
  gender?: string;
  price_min?: string;
  price_max?: string;
  condition?: string;
  size_value?: string;
  size_value2?: string;
  size_system?: string;
  size_group?: string;
  sort_by?: ProductSortField;
  sort_order?: SortOrder;
}

export interface AvailableSizesResponse {
  size_group: string;
  values: string[];
  values2?: string[];
  systems?: string[];
  widths?: string[];
}

export interface ProductFilterOptions {
  colors: string[];
  conditions: string[];
  materials: string[];
  price_min: string | null;
  price_max: string | null;
  brand_ids: number[];
  category_ids: number[];
}
