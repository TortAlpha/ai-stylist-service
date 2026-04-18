import {
  ChangeDetectionStrategy,
  Component,
  OnDestroy,
  OnInit,
  inject,
} from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { FormsModule } from '@angular/forms';
import { TranslateModule, TranslateService } from '@ngx-translate/core';
import { TableModule } from 'primeng/table';
import { ButtonModule } from 'primeng/button';
import { InputTextModule } from 'primeng/inputtext';
import { TagModule } from 'primeng/tag';
import { ConfirmDialogModule } from 'primeng/confirmdialog';
import { ToastModule } from 'primeng/toast';
import { ConfirmationService, MessageService } from 'primeng/api';
import { Subscription } from 'rxjs';
import { AdminBrandsStore } from '../../../store/admin-brands.store';
import { Brand } from '../../../core/models/brand.model';
import { PaginationComponent } from '../../../shared/components/pagination/pagination.component';

@Component({
  selector: 'app-brand-list',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [
    FormsModule,
    TranslateModule,
    TableModule,
    ButtonModule,
    InputTextModule,
    TagModule,
    ConfirmDialogModule,
    ToastModule,
    PaginationComponent,
  ],
  templateUrl: './brand-list.component.html',
  styleUrl: './brand-list.component.scss',
  providers: [ConfirmationService, MessageService],
})
export class BrandListComponent implements OnInit, OnDestroy {
  protected readonly store = inject(AdminBrandsStore);
  private readonly router = inject(Router);
  private readonly route = inject(ActivatedRoute);
  private readonly confirmationService = inject(ConfirmationService);
  private readonly messageService = inject(MessageService);
  private readonly translate = inject(TranslateService);

  filterSearch = '';
  viewMode: 'table' | 'cards' = 'cards';
  private searchDebounceTimer: ReturnType<typeof setTimeout> | null = null;
  private langChangeSub: Subscription | null = null;

  ngOnInit(): void {
    this.langChangeSub = this.translate.onLangChange.subscribe(() => {
      // options are localized via i18n pipe, but keep subscription to be consistent
    });
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

  onSearchInput(): void {
    if (this.searchDebounceTimer) {
      clearTimeout(this.searchDebounceTimer);
    }
    this.searchDebounceTimer = setTimeout(() => {
      this.applyFilters();
    }, 300);
  }

  get viewToggleLabel(): string {
    return this.viewMode === 'table'
      ? this.t('admin.products.list.viewCards')
      : this.t('admin.products.list.viewTable');
  }

  get viewToggleIcon(): string {
    return this.viewMode === 'table' ? 'pi pi-th-large' : 'pi pi-table';
  }

  get activeFilterCount(): number {
    return [this.filterSearch.trim().length > 0].filter(Boolean).length;
  }

  clearFilters(): void {
    this.filterSearch = '';
    void this.store.setFilters({});
    this.syncQueryParams(1);
  }

  toggleViewMode(): void {
    this.viewMode = this.viewMode === 'table' ? 'cards' : 'table';
    this.syncQueryParams();
  }

  goToCreate(): void {
    this.router.navigate(['/admin/brands/new']);
  }

  goToEdit(brandId: number): void {
    this.router.navigate(['/admin/brands', brandId, 'edit']);
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

  confirmDelete(brand: Brand): void {
    this.confirmationService.confirm({
      header: this.t('admin.brands.list.confirmDeleteTitle'),
      message: this.t('admin.brands.list.confirmDeleteMessage', undefined, { name: brand.name }),
      icon: 'pi pi-exclamation-triangle',
      acceptButtonStyleClass: 'p-button-danger',
      accept: () => void this.deleteBrand(brand),
    });
  }

  getTierLabel(tier: string): string {
    return this.t(`brandTiers.${tier}`, tier);
  }

  getTierSeverity(tier: string): 'info' | 'success' | 'warn' | 'secondary' {
    switch (tier) {
      case 'luxury':
        return 'warn';
      case 'premium':
        return 'success';
      case 'mass':
        return 'info';
      default:
        return 'secondary';
    }
  }

  formatDate(value: string | null): string {
    if (!value) return '—';
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return '—';
    return date.toLocaleDateString();
  }

  private applyFilters(): void {
    void this.store.setFilters({
      search: this.filterSearch.trim() || undefined,
    });
    this.syncQueryParams(1);
  }

  private async initializeFromQuery(): Promise<void> {
    const { page, perPage } = this.applyStateFromQuery();

    if (perPage !== this.store.perPage()) {
      await this.store.setPage(1, perPage);
    }

    await this.store.setFilters({
      search: this.filterSearch.trim() || undefined,
    });

    if (page > 1) {
      await this.store.setPage(page, perPage);
    }
  }

  private applyStateFromQuery(): { page: number; perPage: number } {
    const params = this.route.snapshot.queryParamMap;
    const search = params.get('search');
    const page = params.get('page');
    const perPage = params.get('per_page');
    const view = params.get('view');

    this.filterSearch = search ?? '';
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
        page: currentPage > 1 ? currentPage : null,
        per_page: currentPerPage !== 20 ? currentPerPage : null,
        view: this.viewMode !== 'cards' ? this.viewMode : null,
      },
      replaceUrl: true,
    });
  }

  private async deleteBrand(brand: Brand): Promise<void> {
    const ok = await this.store.deleteBrand(brand.id);
    if (ok) {
      this.messageService.add({
        severity: 'success',
        summary: this.t('admin.brands.list.deletedSummary'),
        detail: this.t('admin.brands.list.deletedDetail'),
        life: 2000,
      });
      await this.store.loadBrands();
    } else {
      this.messageService.add({
        severity: 'error',
        summary: this.t('admin.brands.list.deleteFailedSummary'),
        detail: this.store.error() ?? this.t('admin.brands.list.deleteFailedDetail'),
      });
    }
  }

  private t(key: string, fallback?: string, params?: Record<string, string | number>): string {
    const translated = this.translate.instant(key, params);
    return translated === key ? (fallback ?? key) : translated;
  }
}
