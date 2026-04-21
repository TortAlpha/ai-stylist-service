import { ImageVariantUrls } from './product.model';

export type ListingStatus = 'active' | 'paused' | 'sold' | 'removed';

export interface Marketplace {
  id: number;
  name: string;
  code: string;
  base_url: string | null;
}

export interface ProductListing {
  id: number;
  product_id: string;
  marketplace_id: number;
  external_id: string | null;
  external_url: string | null;
  listing_price: string | null;
  sold_price: string | null;
  currency: string | null;
  status: ListingStatus;
  listed_at: string;
  updated_at: string;
}

export interface ListingWithProduct extends ProductListing {
  product_name: string;
  preview_url: ImageVariantUrls | null;
}

export interface CreateListingRequest {
  product_id: string;
  marketplace_id: number;
  external_id?: string;
  external_url?: string;
  listing_price?: string;
  currency?: string;
}

export interface UpdateListingStatusRequest {
  status: string;
  sold_price?: string;
}
