import { ComponentFixture, TestBed } from '@angular/core/testing';
import { provideNoopAnimations } from '@angular/platform-browser/animations';
import { ActivatedRoute, Router, convertToParamMap, provideRouter } from '@angular/router';
import { TranslateModule } from '@ngx-translate/core';
import { Subject, of } from 'rxjs';
import { describe, expect, it, beforeEach, vi } from 'vitest';
import { BrandListComponent } from './brand-list.component';
import { BrandApiService } from '../../../core/services/brand-api.service';

const NOW = '2026-01-01T00:00:00.000Z';

function makeBrandApi() {
  const search = vi.fn().mockReturnValue(of({
    success: true,
    data: { items: [], total: 0, page: 1, per_page: 20, total_pages: 0 },
    error: null,
  }));
  const deleteFn = vi.fn().mockReturnValue(of({ success: true, data: null, error: null }));
  return { search, delete: deleteFn } as any;
}

function routeWith(queryParams: Record<string, string>) {
  return {
    snapshot: { queryParamMap: convertToParamMap(queryParams) },
  };
}

describe('BrandListComponent', () => {
  let fixture: ComponentFixture<BrandListComponent>;
  let component: BrandListComponent;
  let brandApi: ReturnType<typeof makeBrandApi>;
  let router: Router;
  let navigateSpy: ReturnType<typeof vi.spyOn>;

  function setup(queryParams: Record<string, string> = {}) {
    brandApi = makeBrandApi();
    const langChange = new Subject<unknown>();
    const translateStub = {
      instant: (key: string) => key,
      onLangChange: langChange,
    };
    TestBed.configureTestingModule({
      imports: [BrandListComponent, TranslateModule.forRoot()],
      providers: [
        provideNoopAnimations(),
        provideRouter([]),
        { provide: BrandApiService, useValue: brandApi },
        { provide: ActivatedRoute, useValue: routeWith(queryParams) },
      ],
    });
    fixture = TestBed.createComponent(BrandListComponent);
    component = fixture.componentInstance;
    router = TestBed.inject(Router);
    navigateSpy = vi.spyOn(router, 'navigate').mockResolvedValue(true);
  }

  it('initializes filter state from query params', () => {
    setup({ search: 'gucci', view: 'table', page: '2', per_page: '50' });
    component.ngOnInit();

    expect(component.filterSearch).toBe('gucci');
    expect(component.viewMode).toBe('table');
  });

  it('applyFilters (after search input) syncs URL with current search term', () => {
    setup();
    component.filterSearch = 'bag';
    // Bypass debounce by calling the filter handler directly
    component.onFilterChange();

    const [, opts] = navigateSpy.mock.calls.at(-1)!;
    expect((opts as any).queryParams.search).toBe('bag');
  });

  it('onSearchInput debounces applyFilters via setTimeout', () => {
    setup();
    const spy = vi.spyOn(component as any, 'applyFilters' as never);
    component.filterSearch = 'bag';
    component.onSearchInput();
    expect(spy).not.toHaveBeenCalled();
  });

  it('clearFilters resets URL and filters', () => {
    setup({ search: 'x' });
    component.filterSearch = 'x';
    component.clearFilters();

    expect(component.filterSearch).toBe('');
    expect(navigateSpy).toHaveBeenCalled();
    const [, opts] = navigateSpy.mock.calls.at(-1)!;
    expect((opts as any).queryParams.search).toBeNull();
  });

  it('toggles view mode and syncs URL', () => {
    setup();
    component.viewMode = 'cards';
    component.toggleViewMode();

    expect(component.viewMode).toBe('table');
    const [, opts] = navigateSpy.mock.calls.at(-1)!;
    expect((opts as any).queryParams.view).toBe('table');
  });

  it('cardPageChange syncs page and per_page only when different from defaults', () => {
    setup();
    component.onCardPageChange({ page: 3, perPage: 50 });

    const [, opts] = navigateSpy.mock.calls.at(-1)!;
    expect((opts as any).queryParams.page).toBe(3);
    expect((opts as any).queryParams.per_page).toBe(50);
  });

  it('getTierSeverity maps tiers to severities', () => {
    setup();
    expect(component.getTierSeverity('luxury')).toBe('warn');
    expect(component.getTierSeverity('premium')).toBe('success');
    expect(component.getTierSeverity('mass')).toBe('info');
    expect(component.getTierSeverity('other')).toBe('secondary');
  });

  it('formatDate returns dash for null/invalid', () => {
    setup();
    expect(component.formatDate(null)).toBe('—');
    expect(component.formatDate('not-a-date')).toBe('—');
    expect(component.formatDate(NOW)).not.toBe('—');
  });
});
