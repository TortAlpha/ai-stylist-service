import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { UserApiService } from './user-api.service';

describe('UserApiService', () => {
  let service: UserApiService;
  let httpMock: HttpTestingController;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [
        UserApiService,
        provideHttpClient(),
        provideHttpClientTesting(),
      ],
    });
    service = TestBed.inject(UserApiService);
    httpMock = TestBed.inject(HttpTestingController);
  });

  afterEach(() => httpMock.verify());

  it('gets user by id', () => {
    service.getUser('u-1').subscribe();
    const req = httpMock.expectOne('/api/users/u-1');
    expect(req.request.method).toBe('GET');
    req.flush({});
  });

  it('patches user', () => {
    service.updateUser('u-1', { name: 'New' } as any).subscribe();
    const req = httpMock.expectOne('/api/users/u-1');
    expect(req.request.method).toBe('PATCH');
    expect(req.request.body).toEqual({ name: 'New' });
    req.flush({});
  });

  it('lists addresses for user', () => {
    service.getAddresses('u-1').subscribe();
    const req = httpMock.expectOne('/api/users/u-1/addresses');
    expect(req.request.method).toBe('GET');
    req.flush([]);
  });

  it('creates address for user', () => {
    const payload = { country: 'RS', city: 'Belgrade' } as any;
    service.createAddress('u-1', payload).subscribe();
    const req = httpMock.expectOne('/api/users/u-1/addresses');
    expect(req.request.method).toBe('POST');
    expect(req.request.body).toEqual(payload);
    req.flush({});
  });

  it('updates address with PATCH', () => {
    service.updateAddress('u-1', 'a-1', { city: 'Novi Sad' } as any).subscribe();
    const req = httpMock.expectOne('/api/users/u-1/addresses/a-1');
    expect(req.request.method).toBe('PATCH');
    req.flush({});
  });

  it('deletes address', () => {
    service.deleteAddress('u-1', 'a-1').subscribe();
    const req = httpMock.expectOne('/api/users/u-1/addresses/a-1');
    expect(req.request.method).toBe('DELETE');
    req.flush(null);
  });
});
