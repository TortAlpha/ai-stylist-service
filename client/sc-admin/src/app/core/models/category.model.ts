export type Gender = 'male' | 'female' | 'unisex' | 'kids';
export type ProductType = 'clothing' | 'footwear' | 'bags' | 'jewelry' | 'accessories';
export type SizeGroup = 'letter' | 'letter_or_numeric' | 'waist_length' | 'shoe' | 'ring' | 'measurement_cm' | 'dimensions' | 'hat' | 'one_size';

export interface CategoryFullResponse {
  id: number;
  name: string;
  code: string;
  parent_id: number | null;
  gender: string;
  product_type: string;
  size_group: string;
}

export interface CategoryResponse {
  name: string;
  parent_category: string | null;
  gender: string;
  size_group: string;
}

export interface CreateCategoryRequest {
  name: string;
  code: string;
  parent_id?: number;
  gender: Gender;
  product_type: ProductType;
  size_group: SizeGroup;
}

export interface UpdateCategoryRequest {
  name?: string;
  code?: string;
  parent_id?: number;
  gender?: Gender;
  product_type?: ProductType;
  size_group?: SizeGroup;
}
