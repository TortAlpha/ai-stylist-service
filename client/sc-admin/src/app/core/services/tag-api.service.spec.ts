import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { TagApiService } from './tag-api.service';

describe('TagApiService', () => {
  let service: TagApiService;
  let httpMock: HttpTestingController;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [
        TagApiService,
        provideHttpClient(),
        provideHttpClientTesting(),
      ],
    });
    service = TestBed.inject(TagApiService);
    httpMock = TestBed.inject(HttpTestingController);
  });

  afterEach(() => httpMock.verify());

  it.each([
    ['getStyles', 'GET', '/api/tags/styles'],
    ['getVibes', 'GET', '/api/tags/vibes'],
    ['getSeasons', 'GET', '/api/tags/seasons'],
  ] as const)('%s hits %s %s', (method, httpMethod, url) => {
    (service[method] as () => any)().subscribe();
    const req = httpMock.expectOne(url);
    expect(req.request.method).toBe(httpMethod);
    req.flush({ success: true, data: [], error: null });
  });

  it.each([
    ['createStyle', '/api/admin/tags/styles'],
    ['createVibe', '/api/admin/tags/vibes'],
    ['createSeason', '/api/admin/tags/seasons'],
  ] as const)('%s posts to %s', (method, url) => {
    (service[method] as (r: { name: string }) => any)({ name: 'new' }).subscribe();
    const req = httpMock.expectOne(url);
    expect(req.request.method).toBe('POST');
    expect(req.request.body).toEqual({ name: 'new' });
    req.flush({ success: true, data: { id: 1, name: 'new' }, error: null });
  });

  it.each([
    ['deleteStyle', '/api/admin/tags/styles/5'],
    ['deleteVibe', '/api/admin/tags/vibes/5'],
    ['deleteSeason', '/api/admin/tags/seasons/5'],
  ] as const)('%s deletes at %s', (method, url) => {
    (service[method] as (id: number) => any)(5).subscribe();
    const req = httpMock.expectOne(url);
    expect(req.request.method).toBe('DELETE');
    req.flush(null);
  });
});
