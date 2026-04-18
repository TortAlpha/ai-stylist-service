import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { ListingApiService } from './listing-api.service';

describe('ListingApiService', () => {
  let service: ListingApiService;
  let httpMock: HttpTestingController;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [
        ListingApiService,
        provideHttpClient(),
        provideHttpClientTesting(),
      ],
    });
    service = TestBed.inject(ListingApiService);
    httpMock = TestBed.inject(HttpTestingController);
  });

  afterEach(() => httpMock.verify());

  it('fetches all listings', () => {
    service.getAll().subscribe();
    const req = httpMock.expectOne('/api/listings');
    expect(req.request.method).toBe('GET');
    req.flush({ success: true, data: { items: [], total: 0, page: 1, per_page: 20, total_pages: 0 }, error: null });
  });

  it('fetches listings by product', () => {
    service.getByProduct('p-1').subscribe();
    const req = httpMock.expectOne('/api/listings/product/p-1');
    expect(req.request.method).toBe('GET');
    req.flush({ success: true, data: { items: [], total: 0, page: 1, per_page: 20, total_pages: 0 }, error: null });
  });

  it('fetches listing by id', () => {
    service.getById(42).subscribe();
    const req = httpMock.expectOne('/api/listings/42');
    expect(req.request.method).toBe('GET');
    req.flush({ success: true, data: {}, error: null });
  });

  it('creates listing with POST', () => {
    const body = { product_id: 'p-1', marketplace_id: 2 };
    service.create(body).subscribe();
    const req = httpMock.expectOne('/api/listings');
    expect(req.request.method).toBe('POST');
    expect(req.request.body).toEqual(body);
    req.flush({ success: true, data: {}, error: null });
  });

  it('updates status with PATCH', () => {
    service.updateStatus(5, { status: 'sold', sold_price: '99.00' }).subscribe();
    const req = httpMock.expectOne('/api/listings/5/status');
    expect(req.request.method).toBe('PATCH');
    expect(req.request.body).toEqual({ status: 'sold', sold_price: '99.00' });
    req.flush({ success: true, data: {}, error: null });
  });

  it('deletes listing', () => {
    service.delete(9).subscribe();
    const req = httpMock.expectOne('/api/listings/9');
    expect(req.request.method).toBe('DELETE');
    req.flush(null);
  });

  it('fetches marketplaces', () => {
    service.getMarketplaces().subscribe();
    const req = httpMock.expectOne('/api/listings/marketplaces');
    expect(req.request.method).toBe('GET');
    req.flush({ success: true, data: [], error: null });
  });
});
