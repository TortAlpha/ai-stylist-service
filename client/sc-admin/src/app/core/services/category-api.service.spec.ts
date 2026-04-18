import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { CategoryApiService } from './category-api.service';

describe('CategoryApiService', () => {
  let service: CategoryApiService;
  let httpMock: HttpTestingController;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [
        CategoryApiService,
        provideHttpClient(),
        provideHttpClientTesting(),
      ],
    });

    service = TestBed.inject(CategoryApiService);
    httpMock = TestBed.inject(HttpTestingController);
  });

  afterEach(() => {
    httpMock.verify();
  });

  it('fetches public categories without parent filter', () => {
    service.getAll().subscribe();
    const req = httpMock.expectOne(r => r.url === '/api/categories');
    expect(req.request.method).toBe('GET');
    expect(req.request.params.keys().length).toBe(0);
    req.flush({ success: true, data: [], error: null });
  });

  it('passes parent_id when provided', () => {
    service.getAll(5).subscribe();
    const req = httpMock.expectOne(r => r.url === '/api/categories');
    expect(req.request.params.get('parent_id')).toBe('5');
    req.flush({ success: true, data: [], error: null });
  });

  it('fetches admin categories at admin endpoint', () => {
    service.getAllAdmin(10).subscribe();
    const req = httpMock.expectOne(r => r.url === '/api/admin/categories');
    expect(req.request.params.get('parent_id')).toBe('10');
    req.flush({ success: true, data: [], error: null });
  });

  it('creates category with POST', () => {
    service.create({ name: 'Bags', parent_id: null, gender: 'unisex', product_type: 'bags', size_group: 'dimensions' } as any).subscribe();
    const req = httpMock.expectOne('/api/admin/categories');
    expect(req.request.method).toBe('POST');
    expect(req.request.body.name).toBe('Bags');
    req.flush({ success: true, data: {}, error: null });
  });

  it('updates category with PUT', () => {
    service.update(4, { name: 'Renamed' } as any).subscribe();
    const req = httpMock.expectOne('/api/admin/categories/4');
    expect(req.request.method).toBe('PUT');
    req.flush({ success: true, data: {}, error: null });
  });

  it('deletes category by id', () => {
    service.delete(9).subscribe();
    const req = httpMock.expectOne('/api/admin/categories/9');
    expect(req.request.method).toBe('DELETE');
    req.flush({ success: true, data: null, error: null });
  });
});
