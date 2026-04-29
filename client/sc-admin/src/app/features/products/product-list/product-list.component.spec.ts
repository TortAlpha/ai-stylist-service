import { ComponentFixture, TestBed } from '@angular/core/testing';
import { provideNoopAnimations } from '@angular/platform-browser/animations';
import { ActivatedRoute, Router, convertToParamMap, provideRouter } from '@angular/router';
import { TranslateModule } from '@ngx-translate/core';
import { TreeNode } from 'primeng/api';
import { Subject, of } from 'rxjs';
import { describe, expect, it, beforeEach, vi } from 'vitest';
import { ProductListComponent } from './product-list.component';
import { ProductApiService } from '../../../core/services/product-api.service';
import { BrandApiService } from '../../../core/services/brand-api.service';
import { CategoryApiService } from '../../../core/services/category-api.service';
import { ImageCompressionService } from '../../../core/services/image-compression.service';

function routeWith(queryParams: Record<string, string>) {
  return { snapshot: { queryParamMap: convertToParamMap(queryParams) } };
}

function findNodeByLabel(nodes: TreeNode[], label: string): TreeNode | null {
  for (const node of nodes) {
    if (node.label === label) return node;
    const child = findNodeByLabel(node.children ?? [], label);
    if (child) return child;
  }
  return null;
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

  function setup(queryParams: Record<string, string> = {}, categories: unknown[] = []) {
    const productApi = {
      getProducts: vi.fn().mockReturnValue(emptyPage()),
      deleteProduct: vi.fn().mockReturnValue(of({ success: true, data: null, error: null })),
      getFilterOptions: vi.fn().mockReturnValue(of({ success: true, data: {}, error: null })),
      getAvailableSizes: vi.fn().mockReturnValue(of({ success: true, data: { size_group: 'letter', values: [] }, error: null })),
    };
    const brandApi = {
      getAll: vi.fn().mockReturnValue(of({ success: true, data: [], error: null })),
      search: vi.fn().mockReturnValue(emptyPage()),
    };
    const categoryApi = {
      getAll: vi.fn().mockReturnValue(of({ success: true, data: categories, error: null })),
    };

    TestBed.configureTestingModule({
      imports: [ProductListComponent, TranslateModule.forRoot()],
      providers: [
        provideNoopAnimations(),
        provideRouter([]),
        { provide: ProductApiService, useValue: productApi },
        { provide: BrandApiService, useValue: brandApi },
        { provide: CategoryApiService, useValue: categoryApi },
        { provide: ImageCompressionService, useValue: { compress: (f: File) => Promise.resolve(f), compressMany: (fs: File[]) => Promise.resolve(fs) } },
        { provide: ActivatedRoute, useValue: routeWith(queryParams) },
      ],
    });

    fixture = TestBed.createComponent(ProductListComponent);
    component = fixture.componentInstance;
    router = TestBed.inject(Router);
    navigateSpy = vi.spyOn(router, 'navigate').mockResolvedValue(true);
    return { productApi };
  }

  it('reads all filters from query params', async () => {
    setup({
      search: 'jacket',
      brand_id: '7',
      status: 'ready',
      condition: 'good',
      gender: 'female',
      color: 'black',
      size_values: 'M,L',
      size_values2: '32,34',
      size_systems: 'EU,US',
      shoe_widths: 'regular,wide',
      sort_by: 'price',
      sort_order: 'asc',
      view: 'table',
    });
    component.ngOnInit();
    await fixture.whenStable();

    expect(component.filterSearch).toBe('jacket');
    expect(component.filterBrandId).toBe(7);
    expect(component.filterStatus).toBe('ready');
    expect(component.filterCondition).toBe('good');
    expect(component.filterGender).toBe('female');
    expect(component.filterColor).toBe('black');
    expect(component.filterSizeValues).toEqual(['M', 'L']);
    expect(component.filterSizeValues2).toEqual(['32', '34']);
    expect(component.filterSizeSystems).toEqual(['EU', 'US']);
    expect(component.filterShoeWidths).toEqual(['regular', 'wide']);
    expect(component.sortBy).toBe('price');
    expect(component.sortOrder).toBe('asc');
    expect(component.viewMode).toBe('table');
  });

  it('ignores invalid sort_by value and falls back to default', async () => {
    setup({ sort_by: 'garbage' });
    component.ngOnInit();
    await fixture.whenStable();
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

  it('toggles filter panel visibility', () => {
    setup();
    expect(component.filtersCollapsed).toBe(false);
    expect(component.filterPanelToggleLabel).toBe('admin.products.list.hideFilters');
    expect(component.pageSizeOptions).toEqual([10, 20, 40]);

    component.toggleFiltersPanel();

    expect(component.filtersCollapsed).toBe(true);
    expect(component.filterPanelToggleLabel).toBe('admin.products.list.showFilters');
    expect(component.pageSizeOptions).toEqual([12, 18, 36]);
  });

  it('does not reload available sizes when only size value changes', async () => {
    vi.useFakeTimers();
    const { productApi } = setup();
    try {
      component.filterCategoryId = 1;
      component.filterSizeValues = ['M'];

      component.onSizeValueFilterChange();
      vi.advanceTimersByTime(120);
      await fixture.whenStable();

      expect(productApi.getProducts).toHaveBeenCalled();
      expect(productApi.getAvailableSizes).not.toHaveBeenCalled();
    } finally {
      vi.useRealTimers();
    }
  });

  it('reloads available sizes when size system changes', async () => {
    const { productApi } = setup();
    component.filterCategoryId = 15;
    component.filterSizeSystems = ['EU'];

    component.onSizeFilterChange();
    await fixture.whenStable();

    expect(productApi.getProducts).toHaveBeenCalled();
    expect(productApi.getAvailableSizes).toHaveBeenCalledWith(
      15,
      undefined,
      undefined,
      undefined,
      undefined,
      undefined,
      undefined,
      'EU',
      undefined,
      undefined,
    );
  });

  it('clearFilters resets filter state and URL params', () => {
    setup();
    component.filterSearch = 'coat';
    component.filterBrandId = 7;
    component.filterCategoryId = 15;
    component.selectedCategoryNode = { key: 'c:15', data: { id: 15 } };
    component.filterStatus = 'ready';
    component.filterCondition = 'good';
    component.filterGender = 'female';
    component.filterColor = 'black';
    component.filterSizeValues = ['M'];
    component.filterSizeValues2 = ['32'];
    component.filterSizeSystems = ['EU'];
    component.filterShoeWidths = ['wide'];
    component.filterPriceMin = '10';
    component.filterPriceMax = '100';
    component.sortBy = 'price';
    component.sortOrder = 'asc';

    component.clearFilters();

    expect(component.activeFilterCount).toBe(0);
    expect(component.filterSizeValues).toEqual([]);
    expect(component.filterSizeValues2).toEqual([]);
    expect(component.filterSizeSystems).toEqual([]);
    expect(component.filterShoeWidths).toEqual([]);
    expect(component.selectedCategoryNode).toBeNull();
    const [, opts] = navigateSpy.mock.calls.at(-1)!;
    const q = (opts as any).queryParams;
    expect(q.search).toBeNull();
    expect(q.category_id).toBeNull();
    expect(q.size_values).toBeNull();
    expect(q.size_values2).toBeNull();
    expect(q.size_systems).toBeNull();
    expect(q.shoe_widths).toBeNull();
    expect(q.sort_by).toBeNull();
    expect(q.sort_order).toBeNull();
  });

  it('category tree hides root category when it duplicates product type', async () => {
    setup({}, [
      {
        id: 1,
        name: 'Bags',
        parent_id: null,
        gender: 'female',
        product_type: 'bags',
        size_group: 'dimensions',
      },
      {
        id: 2,
        name: 'Crossbody Bags',
        parent_id: 1,
        gender: 'female',
        product_type: 'bags',
        size_group: 'dimensions',
      },
    ]);
    component.ngOnInit();
    await fixture.whenStable();

    const typeNode = findNodeByLabel(component.categoryTreeNodes, 'Bags');

    expect(typeNode?.children?.map(node => node.label)).toEqual(['Crossbody Bags']);
    component.filterCategoryId = 2;
    expect(component.selectedCategoryPath).toBe('Female / Bags / Crossbody Bags');
  });
});
