import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { PurchaseLocationApiService } from './purchase-location-api.service';

describe('PurchaseLocationApiService', () => {
  let service: PurchaseLocationApiService;
  let httpMock: HttpTestingController;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [
        PurchaseLocationApiService,
        provideHttpClient(),
        provideHttpClientTesting(),
      ],
    });

    service = TestBed.inject(PurchaseLocationApiService);
    httpMock = TestBed.inject(HttpTestingController);
  });

  afterEach(() => {
    httpMock.verify();
  });

  it('requests purchase locations list', () => {
    service.getAll().subscribe(response => {
      expect(response.success).toBe(true);
      expect(response.data?.length).toBe(1);
      expect(response.data?.[0].name).toBe('Belgrade Warehouse');
    });

    const req = httpMock.expectOne('/api/admin/purchase-locations');
    expect(req.request.method).toBe('GET');
    req.flush({
      success: true,
      data: [{ id: 1, name: 'Belgrade Warehouse', created_at: '2026-01-01T00:00:00.000Z' }],
      error: null,
    });
  });
});
