import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { BrandApiService } from './brand-api.service';

const NOW = '2026-01-01T00:00:00.000Z';

describe('BrandApiService', () => {
  let service: BrandApiService;
  let httpMock: HttpTestingController;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [
        BrandApiService,
        provideHttpClient(),
        provideHttpClientTesting(),
      ],
    });

    service = TestBed.inject(BrandApiService);
    httpMock = TestBed.inject(HttpTestingController);
  });

  afterEach(() => {
    httpMock.verify();
  });

  it('gets all brands from public endpoint', () => {
    service.getAll().subscribe(res => {
      expect(res.data?.length).toBe(1);
    });

    const req = httpMock.expectOne('/api/brands');
    expect(req.request.method).toBe('GET');
    req.flush({
      success: true,
      data: [{ id: 1, name: 'Acme', code: 'ACME', tier: 'premium', country: 'RS', created_at: NOW }],
      error: null,
    });
  });

  it('passes search pagination params to admin search', () => {
    service.search('gucci', 2, 50).subscribe();

    const req = httpMock.expectOne(r => r.url === '/api/admin/brands');
    expect(req.request.method).toBe('GET');
    expect(req.request.params.get('search')).toBe('gucci');
    expect(req.request.params.get('page')).toBe('2');
    expect(req.request.params.get('per_page')).toBe('50');
    req.flush({ success: true, data: { items: [], total: 0, page: 2, per_page: 50, total_pages: 0 }, error: null });
  });

  it('omits params for empty search', () => {
    service.search().subscribe();

    const req = httpMock.expectOne(r => r.url === '/api/admin/brands');
    expect(req.request.params.keys().length).toBe(0);
    req.flush({ success: true, data: { items: [], total: 0, page: 1, per_page: 20, total_pages: 0 }, error: null });
  });

  it('creates brand with POST', () => {
    service.create({ name: 'Gucci', code: 'GUCCI', tier: 'luxury' }).subscribe();

    const req = httpMock.expectOne('/api/admin/brands');
    expect(req.request.method).toBe('POST');
    expect(req.request.body).toEqual({ name: 'Gucci', code: 'GUCCI', tier: 'luxury' });
    req.flush({ success: true, data: { id: 2, name: 'Gucci', code: 'GUCCI', tier: 'luxury', country: null, created_at: NOW }, error: null });
  });

  it('updates brand with PUT', () => {
    service.update(3, { name: 'Renamed' }).subscribe();

    const req = httpMock.expectOne('/api/admin/brands/3');
    expect(req.request.method).toBe('PUT');
    expect(req.request.body).toEqual({ name: 'Renamed' });
    req.flush({ success: true, data: { id: 3, name: 'Renamed', code: 'X', tier: 'mass', country: null, created_at: NOW }, error: null });
  });

  it('deletes brand by id', () => {
    service.delete(7).subscribe();

    const req = httpMock.expectOne('/api/admin/brands/7');
    expect(req.request.method).toBe('DELETE');
    req.flush({ success: true, data: null, error: null });
  });
});
