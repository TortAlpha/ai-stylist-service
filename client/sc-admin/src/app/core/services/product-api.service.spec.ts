import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { ProductApiService } from './product-api.service';
import { ImageCompressionService } from './image-compression.service';
import { CreateProductRequest } from '../models/product.model';

class PassThroughCompression {
  compress(file: File): Promise<File> { return Promise.resolve(file); }
  compressMany(files: File[]): Promise<File[]> { return Promise.all(files.map(f => this.compress(f))); }
}

describe('ProductApiService', () => {
  let service: ProductApiService;
  let httpMock: HttpTestingController;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [
        ProductApiService,
        provideHttpClient(),
        provideHttpClientTesting(),
        { provide: ImageCompressionService, useClass: PassThroughCompression },
      ],
    });

    service = TestBed.inject(ProductApiService);
    httpMock = TestBed.inject(HttpTestingController);
  });

  afterEach(() => {
    httpMock.verify();
  });

  it('builds query params for products list', () => {
    service.getProducts({
      page: 2,
      per_page: 20,
      search: 'jacket',
      brand_id: 7,
      status: 'ready',
      color: 'black',
      size_values: 'M,L',
      size_systems: 'EU,US',
      shoe_widths: 'regular,wide',
      sort_by: 'name',
      sort_order: 'asc',
    }).subscribe();

    const req = httpMock.expectOne(request => request.url === '/api/admin/products');
    expect(req.request.method).toBe('GET');
    expect(req.request.params.get('page')).toBe('2');
    expect(req.request.params.get('per_page')).toBe('20');
    expect(req.request.params.get('search')).toBe('jacket');
    expect(req.request.params.get('brand_id')).toBe('7');
    expect(req.request.params.get('status')).toBe('ready');
    expect(req.request.params.get('color')).toBe('black');
    expect(req.request.params.get('size_values')).toBe('M,L');
    expect(req.request.params.get('size_systems')).toBe('EU,US');
    expect(req.request.params.get('shoe_widths')).toBe('regular,wide');
    expect(req.request.params.get('sort_by')).toBe('name');
    expect(req.request.params.get('sort_order')).toBe('asc');

    req.flush({
      success: true,
      data: { items: [], total: 0, page: 2, per_page: 20, total_pages: 0 },
      error: null,
    });
  });

  it('builds optional params for available sizes', () => {
    service.getAvailableSizes(15, 'shoe', 7, 'female', 'ready', 'excellent', 'black', 'EU,US', '10', '500').subscribe();

    const req = httpMock.expectOne(request => request.url === '/api/admin/products/available-sizes');
    expect(req.request.method).toBe('GET');
    expect(req.request.params.get('category_id')).toBe('15');
    expect(req.request.params.get('size_group')).toBe('shoe');
    expect(req.request.params.get('brand_id')).toBe('7');
    expect(req.request.params.get('gender')).toBe('female');
    expect(req.request.params.get('status')).toBe('ready');
    expect(req.request.params.get('condition')).toBe('excellent');
    expect(req.request.params.get('color')).toBe('black');
    expect(req.request.params.get('size_systems')).toBe('EU,US');
    expect(req.request.params.get('price_min')).toBe('10');
    expect(req.request.params.get('price_max')).toBe('500');

    req.flush({
      success: true,
      data: { size_group: 'shoe', values: ['42', '43'], widths: ['narrow', 'regular'] },
      error: null,
    });
  });

  it('sends expected_version on delete', () => {
    service.deleteProduct('prod-1', 5).subscribe();

    const req = httpMock.expectOne(request => request.url === '/api/admin/products/prod-1');
    expect(req.request.method).toBe('DELETE');
    expect(req.request.params.get('expected_version')).toBe('5');

    req.flush({
      success: true,
      data: null,
      error: null,
    });
  });

  it('creates product payload as multipart form data', async () => {
    const request: CreateProductRequest = {
      name: 'Test Product',
      brand_id: 1,
      category_id: 2,
      status: 'ready',
      purchase_price: '100.00',
      currency: 'EUR',
      details: {
        condition: 'excellent',
        size: { size_value: 'M' },
        type_details: { product_type: 'clothing', fit: 'regular' },
      },
      style_tag_ids: [10],
      vibe_tag_ids: [20],
      season_ids: [30],
    };
    const preview = new File(['preview'], 'preview.jpg', { type: 'image/jpeg' });
    const image1 = new File(['image-1'], '1.jpg', { type: 'image/jpeg' });
    const image2 = new File(['image-2'], '2.jpg', { type: 'image/jpeg' });

    service.createProduct(request, preview, [image1, image2]).subscribe();

    // buildProductFormData is async (awaits compression) — wait one macrotask
    await new Promise(resolve => setTimeout(resolve, 0));

    const req = httpMock.expectOne('/api/admin/products');
    expect(req.request.method).toBe('POST');
    expect(req.request.body instanceof FormData).toBe(true);

    const body = req.request.body as FormData;
    expect(body.get('metadata')).toBe(JSON.stringify(request));
    expect(body.get('preview')).toBe(preview);
    expect(body.getAll('images')).toEqual([image1, image2]);

    req.flush({
      success: true,
      data: {
        id: 'prod-1',
        sku: 'SKU-1',
        name: 'Test Product',
        purchase_price: '100.00',
        purchase_location: null,
        currency: 'EUR',
        ai_notes: null,
        preview_url: null,
        image_urls: [],
        brand: { id: 1, name: 'Brand', tier: 'premium' },
        brand_id: 1,
        product_type: 'clothing',
        category: {
          name: 'Outerwear',
          parent_category: null,
          gender: 'unisex',
          size_group: 'clothing',
        },
        category_id: 2,
        status: 'ready',
        version: 1,
        details: {
          material: null,
          condition: 'excellent',
          color: null,
          year_of_release: null,
          is_vintage: false,
          is_collab: false,
          collab_name: null,
          is_limited_edition: false,
          special_notes: null,
        },
        size: { size_group: 'clothing', size_value: 'M' },
        type_details: { type: 'clothing', fit: 'regular' },
        tags: { styles: [], vibes: [], seasons: [] },
        created_at: '2026-01-01T00:00:00.000Z',
        updated_at: '2026-01-01T00:00:00.000Z',
      },
      error: null,
    });
  });
});
