import {
  ChangeDetectionStrategy,
  Component,
  inject,
  OnDestroy,
  OnInit,
} from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { FormsModule } from '@angular/forms';
import { TranslateModule, TranslateService } from '@ngx-translate/core';
import { TableModule } from 'primeng/table';
import { ButtonModule } from 'primeng/button';
import { SelectModule } from 'primeng/select';
import { InputTextModule } from 'primeng/inputtext';
import { TagModule } from 'primeng/tag';
import { ConfirmDialogModule } from 'primeng/confirmdialog';
import { ToastModule } from 'primeng/toast';
import { ConfirmationService, MessageService } from 'primeng/api';
import { Subscription, firstValueFrom } from 'rxjs';
import { AdminProductsStore } from '../../../store/admin-products.store';
import { BrandStore } from '../../../store/brand.store';
import { ProductApiService } from '../../../core/services/product-api.service';
import { ProductPreviewResponse } from '../../../core/models/product.model';
import { PaginationComponent } from '../../../shared/components/pagination/pagination.component';

@Component({
  selector: 'app-product-list',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [
    FormsModule,
    TranslateModule,
    TableModule,
    ButtonModule,
    SelectModule,
    InputTextModule,
    TagModule,
    ConfirmDialogModule,
    ToastModule,
    PaginationComponent,
  ],
  templateUrl: './product-list.component.html',
  styleUrl: './product-list.component.scss',
  providers: [BrandStore, ConfirmationService, MessageService],
})
export class ProductListComponent implements OnInit, OnDestroy {
  protected readonly store = inject(AdminProductsStore);
  private readonly brandStore = inject(BrandStore);
  private readonly router = inject(Router);
  private readonly route = inject(ActivatedRoute);
  private readonly productApi = inject(ProductApiService);
  private readonly confirmationService = inject(ConfirmationService);
  private readonly messageService = inject(MessageService);
  private readonly translate = inject(TranslateService);

  filterBrandId: number | null = null;
  filterProductType: string | null = null;
  filterStatus: string | null = null;
  filterCondition: string | null = null;
  filterGender: string | null = null;
  filterSearch = '';
  sortBy: 'price' | 'created_at' | 'updated_at' | 'name' = 'created_at';
  sortOrder: 'asc' | 'desc' = 'desc';
  viewMode: 'table' | 'cards' = 'cards';
  private searchDebounceTimer: ReturnType<typeof setTimeout> | null = null;
  private langChangeSub: Subscription | null = null;

  typeOptions: Array<{ label: string; value: string }> = [];

  statusOptions: Array<{ label: string; value: string }> = [];

  conditionOptions: Array<{ label: string; value: string }> = [];

  genderOptions: Array<{ label: string; value: string }> = [];

  sortByOptions: Array<{ label: string; value: 'price' | 'created_at' | 'updated_at' | 'name' }> = [];

  get brandOptions(): { label: string; value: number }[] {
    return this.brandStore.brands().map(b => ({ label: b.name, value: b.id }));
  }

  ngOnInit(): void {
    this.buildLocalizedOptions();
    this.langChangeSub = this.translate.onLangChange.subscribe(() => {
      this.buildLocalizedOptions();
    });
    this.brandStore.loadBrands();
    void this.initializeFromQuery();
  }

  ngOnDestroy(): void {
    if (this.searchDebounceTimer) {
      clearTimeout(this.searchDebounceTimer);
    }
    this.langChangeSub?.unsubscribe();
  }

  onFilterChange(): void {
    this.applyFilters();
  }

  toggleSortOrder(): void {
    this.sortOrder = this.sortOrder === 'desc' ? 'asc' : 'desc';
    this.applyFilters();
  }

  get sortOrderIcon(): string {
    return this.sortOrder === 'asc' ? 'pi pi-sort-amount-up-alt' : 'pi pi-sort-amount-down-alt';
  }

  get sortOrderAriaLabel(): string {
    return this.sortOrder === 'asc'
      ? this.t('admin.products.list.sortOrderAsc')
      : this.t('admin.products.list.sortOrderDesc');
  }

  get viewToggleLabel(): string {
    return this.viewMode === 'table'
      ? this.t('admin.products.list.viewCards')
      : this.t('admin.products.list.viewTable');
  }

  get viewToggleIcon(): string {
    return this.viewMode === 'table' ? 'pi pi-th-large' : 'pi pi-table';
  }

  private buildLocalizedOptions(): void {
    this.typeOptions = [
      { label: this.t('productTypes.clothing'), value: 'clothing' },
      { label: this.t('productTypes.footwear'), value: 'footwear' },
      { label: this.t('productTypes.bags'), value: 'bags' },
      { label: this.t('productTypes.jewelry'), value: 'jewelry' },
      { label: this.t('productTypes.accessories'), value: 'accessories' },
    ];

    this.statusOptions = [
      { label: this.t('statuses.intake'), value: 'intake' },
      { label: this.t('statuses.inspection'), value: 'inspection' },
      { label: this.t('statuses.rejected'), value: 'rejected' },
      { label: this.t('statuses.preparation'), value: 'preparation' },
      { label: this.t('statuses.photo_queue'), value: 'photo_queue' },
      { label: this.t('statuses.photo_done'), value: 'photo_done' },
      { label: this.t('statuses.ready'), value: 'ready' },
      { label: this.t('statuses.reserved'), value: 'reserved' },
      { label: this.t('statuses.sold'), value: 'sold' },
      { label: this.t('statuses.returned'), value: 'returned' },
    ];

    this.conditionOptions = [
      { label: this.t('conditions.new_with_tags'), value: 'new_with_tags' },
      { label: this.t('conditions.excellent'), value: 'excellent' },
      { label: this.t('conditions.good'), value: 'good' },
      { label: this.t('conditions.fair'), value: 'fair' },
    ];

    this.genderOptions = [
      { label: this.t('genders.male'), value: 'male' },
      { label: this.t('genders.female'), value: 'female' },
      { label: this.t('genders.unisex'), value: 'unisex' },
    ];

    this.sortByOptions = [
      { label: this.t('admin.products.list.sort.createdAt'), value: 'created_at' },
      { label: this.t('admin.products.list.sort.updatedAt'), value: 'updated_at' },
      { label: this.t('admin.products.list.sort.price'), value: 'price' },
      { label: this.t('admin.products.list.sort.name'), value: 'name' },
    ];
  }

  private async initializeFromQuery(): Promise<void> {
    const { page, perPage } = this.applyStateFromQuery();

    if (perPage !== this.store.perPage()) {
      await this.store.setPage(1, perPage);
    }

    await this.store.setFilters({
      search: this.filterSearch.trim() || undefined,
      brand_id: this.filterBrandId ?? undefined,
      product_type: this.filterProductType ?? undefined,
      status: this.filterStatus ?? undefined,
      condition: this.filterCondition ?? undefined,
      gender: this.filterGender ?? undefined,
      sort_by: this.sortBy,
      sort_order: this.sortOrder,
    });

    if (page > 1) {
      await this.store.setPage(page, perPage);
    }
  }

  private applyFilters(): void {
    void this.store.setFilters({
      search: this.filterSearch.trim() || undefined,
      brand_id: this.filterBrandId ?? undefined,
      product_type: this.filterProductType ?? undefined,
      status: this.filterStatus ?? undefined,
      condition: this.filterCondition ?? undefined,
      gender: this.filterGender ?? undefined,
      sort_by: this.sortBy,
      sort_order: this.sortOrder,
    });
    this.syncQueryParams(1);
  }

  clearFilters(): void {
    this.filterSearch = '';
    this.filterBrandId = null;
    this.filterProductType = null;
    this.filterStatus = null;
    this.filterCondition = null;
    this.filterGender = null;
    this.sortBy = 'created_at';
    this.sortOrder = 'desc';
    void this.store.setFilters({
      sort_by: this.sortBy,
      sort_order: this.sortOrder,
    });
    this.syncQueryParams(1);
  }

  onPageChange(event: any): void {
    const page = Math.floor(event.first / event.rows) + 1;
    const perPage = Number(event.rows) || this.store.perPage();
    void this.store.setPage(page, perPage);
    this.syncQueryParams(page, perPage);
  }

  onCardPageChange(event: { page: number; perPage: number }): void {
    const page = Math.max(1, Number(event.page) || 1);
    const perPage = Math.max(1, Number(event.perPage) || this.store.perPage());
    void this.store.setPage(page, perPage);
    this.syncQueryParams(page, perPage);
  }

  goToPage(page: number): void {
    const safePage = Math.max(1, page);
    const perPage = this.store.perPage();
    void this.store.setPage(safePage, perPage);
    this.syncQueryParams(safePage, perPage);
  }

  onSearchInput(): void {
    if (this.searchDebounceTimer) {
      clearTimeout(this.searchDebounceTimer);
    }
    this.searchDebounceTimer = setTimeout(() => {
      this.applyFilters();
    }, 300);
  }

  goToCreate(): void {
    this.router.navigate(['/admin/products/new']);
  }

  toggleViewMode(): void {
    this.viewMode = this.viewMode === 'table' ? 'cards' : 'table';
    this.syncQueryParams();
  }

  goToEdit(productId: ProductPreviewResponse['id']): void {
    this.router.navigate(['/admin/products', productId, 'edit']);
  }

  goToDetail(productId: ProductPreviewResponse['id']): void {
    this.router.navigate(['/admin/products', productId]);
  }

  get activeFilterCount(): number {
    return [
      this.filterSearch.trim().length > 0,
      this.filterBrandId !== null,
      this.filterProductType !== null,
      this.filterStatus !== null,
      this.filterCondition !== null,
      this.filterGender !== null,
      this.sortBy !== 'created_at',
      this.sortOrder !== 'desc',
    ].filter(Boolean).length;
  }

  private applyStateFromQuery(): { page: number; perPage: number } {
    const params = this.route.snapshot.queryParamMap;
    const search = params.get('search');
    const brandId = params.get('brand_id');
    const page = params.get('page');
    const perPage = params.get('per_page');
    const sortBy = params.get('sort_by');
    const sortOrder = params.get('sort_order');
    const view = params.get('view');

    this.filterSearch = search ?? '';
    this.filterBrandId = brandId ? Number(brandId) : null;
    this.filterProductType = params.get('product_type');
    this.filterStatus = params.get('status');
    this.filterCondition = params.get('condition');
    this.filterGender = params.get('gender');
    this.sortBy = this.isSortField(sortBy) ? sortBy : 'created_at';
    this.sortOrder = sortOrder === 'asc' ? 'asc' : 'desc';
    this.viewMode = view === 'table' ? 'table' : 'cards';

    const pageNum = page ? Number(page) : 1;
    const perPageNum = perPage ? Number(perPage) : this.store.perPage();
    const normalizedPage = Number.isFinite(pageNum) && pageNum > 0 ? pageNum : 1;
    const normalizedPerPage =
      Number.isFinite(perPageNum) && perPageNum > 0 ? perPageNum : this.store.perPage();

    return { page: normalizedPage, perPage: normalizedPerPage };
  }

  private syncQueryParams(page?: number, perPage?: number): void {
    const currentPage = page ?? this.store.page();
    const currentPerPage = perPage ?? this.store.perPage();
    this.router.navigate([], {
      relativeTo: this.route,
      queryParams: {
        search: this.filterSearch.trim() || null,
        brand_id: this.filterBrandId ?? null,
        product_type: this.filterProductType ?? null,
        status: this.filterStatus ?? null,
        condition: this.filterCondition ?? null,
        gender: this.filterGender ?? null,
        sort_by: this.sortBy !== 'created_at' ? this.sortBy : null,
        sort_order: this.sortOrder !== 'desc' ? this.sortOrder : null,
        page: currentPage > 1 ? currentPage : null,
        per_page: currentPerPage !== 20 ? currentPerPage : null,
        view: this.viewMode !== 'cards' ? this.viewMode : null,
      },
      replaceUrl: true,
    });
  }

  private isSortField(value: string | null): value is 'price' | 'created_at' | 'updated_at' | 'name' {
    return value === 'price' || value === 'created_at' || value === 'updated_at' || value === 'name';
  }

  goToListings(productId: string): void {
    this.router.navigate(['/admin/products', productId, 'listings']);
  }

  goToHistory(productId: string): void {
    this.router.navigate(['/admin/products', productId, 'history']);
  }

  goToImages(productId: string): void {
    this.router.navigate(['/admin/products', productId, 'images']);
  }

  confirmDelete(product: ProductPreviewResponse): void {
    this.confirmationService.confirm({
      header: this.t('admin.products.list.confirmDeleteTitle'),
      message: this.t('admin.products.list.confirmDeleteMessage', undefined, { name: product.name }),
      icon: 'pi pi-exclamation-triangle',
      acceptButtonStyleClass: 'p-button-danger',
      accept: () => void this.deleteProduct(product.id),
    });
  }

  getStatusLabel(status?: string): string {
    if (!status) {
      return '—';
    }
    return this.t(`statuses.${status}`, status);
  }

  getConditionLabel(condition?: string): string {
    if (!condition) {
      return '—';
    }
    return this.t(`conditions.${condition}`, condition);
  }

  getTypeLabel(type: string): string {
    return this.t(`productTypes.${type}`, type);
  }

  private t(key: string, fallback?: string, params?: Record<string, string | number>): string {
    const translated = this.translate.instant(key, params);
    return translated === key ? (fallback ?? key) : translated;
  }

  private async deleteProduct(productId: string): Promise<void> {
    try {
      const productRes = await firstValueFrom(this.productApi.getProduct(productId));
      if (!productRes.success || !productRes.data) {
        this.messageService.add({
          severity: 'error',
          summary: this.t('admin.products.list.deleteFailedSummary'),
          detail: productRes.error ?? this.t('admin.products.list.loadVersionFailedDetail'),
        });
        return;
      }

      const deleteRes = await firstValueFrom(
        this.productApi.deleteProduct(productId, productRes.data.version),
      );
      if (!deleteRes.success) {
        this.messageService.add({
          severity: 'error',
          summary: this.t('admin.products.list.deleteFailedSummary'),
          detail: deleteRes.error ?? this.t('admin.products.list.deleteFailedDetail'),
        });
        return;
      }

      this.messageService.add({
        severity: 'success',
        summary: this.t('admin.products.list.deletedSummary'),
        detail: this.t('admin.products.list.deletedDetail'),
        life: 2000,
      });
      await this.store.loadProducts();
    } catch (error: any) {
      this.messageService.add({
        severity: 'error',
        summary: this.t('admin.products.list.deleteFailedSummary'),
        detail: error?.error?.error ?? this.t('admin.products.list.deleteFailedDetail'),
      });
    }
  }
}
