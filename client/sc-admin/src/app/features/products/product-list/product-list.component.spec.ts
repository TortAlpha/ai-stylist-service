import { ComponentFixture, TestBed } from '@angular/core/testing';
import { provideNoopAnimations } from '@angular/platform-browser/animations';
import { ActivatedRoute, Router, convertToParamMap, provideRouter } from '@angular/router';
import { TranslateModule } from '@ngx-translate/core';
import { Subject, of } from 'rxjs';
import { describe, expect, it, beforeEach, vi } from 'vitest';
import { ProductListComponent } from './product-list.component';
import { ProductApiService } from '../../../core/services/product-api.service';
import { BrandApiService } from '../../../core/services/brand-api.service';
import { ImageCompressionService } from '../../../core/services/image-compression.service';

function routeWith(queryParams: Record<string, string>) {
  return { snapshot: { queryParamMap: convertToParamMap(queryParams) } };
}

const emptyPage = (overrides: Record<string, unknown> = {}) => of({
  success: true,
  data: { items: [], total: 0, page: 1, per_page: 20, total_pages: 0, ...overrides },
  error: null,
});

describe('ProductListComponent', () => {
  let fixture: ComponentFixture<ProductListComponent>;
  let component: ProductListComponent;
  let router: Router;
  let navigateSpy: ReturnType<typeof vi.spyOn>;

  function setup(queryParams: Record<string, string> = {}) {
    const productApi = {
      getProducts: vi.fn().mockReturnValue(emptyPage()),
      deleteProduct: vi.fn().mockReturnValue(of({ success: true, data: null, error: null })),
      getFilterOptions: vi.fn().mockReturnValue(of({ success: true, data: {}, error: null })),
    };
    const brandApi = {
      getAll: vi.fn().mockReturnValue(of({ success: true, data: [], error: null })),
      search: vi.fn().mockReturnValue(emptyPage()),
    };

    TestBed.configureTestingModule({
      imports: [ProductListComponent, TranslateModule.forRoot()],
      providers: [
        provideNoopAnimations(),
        provideRouter([]),
        { provide: ProductApiService, useValue: productApi },
        { provide: BrandApiService, useValue: brandApi },
        { provide: ImageCompressionService, useValue: { compress: (f: File) => Promise.resolve(f), compressMany: (fs: File[]) => Promise.resolve(fs) } },
        { provide: ActivatedRoute, useValue: routeWith(queryParams) },
      ],
    });

    fixture = TestBed.createComponent(ProductListComponent);
    component = fixture.componentInstance;
    router = TestBed.inject(Router);
    navigateSpy = vi.spyOn(router, 'navigate').mockResolvedValue(true);
  }

  it('reads all filters from query params', () => {
    setup({
      search: 'jacket',
      brand_id: '7',
      product_type: 'clothing',
      status: 'ready',
      condition: 'good',
      gender: 'female',
      sort_by: 'price',
      sort_order: 'asc',
      view: 'table',
    });
    component.ngOnInit();

    expect(component.filterSearch).toBe('jacket');
    expect(component.filterBrandId).toBe(7);
    expect(component.filterProductType).toBe('clothing');
    expect(component.filterStatus).toBe('ready');
    expect(component.filterCondition).toBe('good');
    expect(component.filterGender).toBe('female');
    expect(component.sortBy).toBe('price');
    expect(component.sortOrder).toBe('asc');
    expect(component.viewMode).toBe('table');
  });

  it('ignores invalid sort_by value and falls back to default', () => {
    setup({ sort_by: 'garbage' });
    component.ngOnInit();
    expect(component.sortBy).toBe('created_at');
  });

  it('toggleSortOrder flips asc/desc and syncs URL', () => {
    setup();
    component.sortOrder = 'desc';
    component.toggleSortOrder();

    expect(component.sortOrder).toBe('asc');
    expect(navigateSpy).toHaveBeenCalled();
    const [, opts] = navigateSpy.mock.calls.at(-1)!;
    expect((opts as any).queryParams.sort_order).toBe('asc');
  });

  it('omits default sort/view/page params from URL', () => {
    setup();
    component.sortOrder = 'desc';
    component.sortBy = 'created_at';
    component.viewMode = 'cards';
    component.filterSearch = '';
    component.onFilterChange();

    const [, opts] = navigateSpy.mock.calls.at(-1)!;
    const q = (opts as any).queryParams;
    expect(q.sort_order).toBeNull();
    expect(q.sort_by).toBeNull();
    expect(q.view).toBeNull();
    expect(q.page).toBeNull();
    expect(q.search).toBeNull();
  });

  it('sortOrderIcon reflects sortOrder', () => {
    setup();
    component.sortOrder = 'asc';
    expect(component.sortOrderIcon).toBe('pi pi-sort-amount-up-alt');
    component.sortOrder = 'desc';
    expect(component.sortOrderIcon).toBe('pi pi-sort-amount-down-alt');
  });
});
