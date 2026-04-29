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
import { MultiSelectModule } from 'primeng/multiselect';
import { TreeSelectModule } from 'primeng/treeselect';
import { InputTextModule } from 'primeng/inputtext';
import { TagModule } from 'primeng/tag';
import { ConfirmDialogModule } from 'primeng/confirmdialog';
import { ToastModule } from 'primeng/toast';
import { ConfirmationService, MessageService, TreeNode } from 'primeng/api';
import { Subscription, firstValueFrom } from 'rxjs';
import { AdminProductsStore } from '../../../store/admin-products.store';
import { BrandStore } from '../../../store/brand.store';
import { CategoryStore } from '../../../store/category.store';
import { ProductFilterOptionsStore } from '../../../store/product-filter-options.store';
import { ProductApiService } from '../../../core/services/product-api.service';
import { ProductPreviewResponse } from '../../../core/models/product.model';
import { CategoryFullResponse } from '../../../core/models/category.model';
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
    MultiSelectModule,
    TreeSelectModule,
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
  protected readonly categoryStore = inject(CategoryStore);
  protected readonly filterOptionsStore = inject(ProductFilterOptionsStore);
  private readonly router = inject(Router);
  private readonly route = inject(ActivatedRoute);
  private readonly productApi = inject(ProductApiService);
  private readonly confirmationService = inject(ConfirmationService);
  private readonly messageService = inject(MessageService);
  private readonly translate = inject(TranslateService);

  filterBrandId: number | null = null;
  filterCategoryId: number | null = null;
  selectedCategoryNode: TreeNode | null = null;
  filterStatus: string | null = null;
  filterCondition: string | null = null;
  filterGender: string | null = null;
  filterColor: string | null = null;
  filterSizeValues: string[] = [];
  filterSizeValues2: string[] = [];
  filterSizeSystems: string[] = [];
  filterShoeWidths: string[] = [];
  filterPriceMin: string | null = null;
  filterPriceMax: string | null = null;
  filterSearch = '';
  sortBy: 'price' | 'created_at' | 'updated_at' | 'name' = 'created_at';
  sortOrder: 'asc' | 'desc' = 'desc';
  viewMode: 'table' | 'cards' = 'cards';
  filtersCollapsed = false;
  private searchDebounceTimer: ReturnType<typeof setTimeout> | null = null;
  private priceDebounceTimer: ReturnType<typeof setTimeout> | null = null;
  private sizeDebounceTimer: ReturnType<typeof setTimeout> | null = null;
  private langChangeSub: Subscription | null = null;
  private readonly categoryNodeCache = new Map<string, TreeNode>();
  private readonly pageSizeOptionsOpen = [10, 20, 40];
  private readonly pageSizeOptionsCollapsed = [12, 18, 36];

  statusOptions: Array<{ label: string; value: string }> = [];

  conditionOptions: Array<{ label: string; value: string }> = [];

  genderOptions: Array<{ label: string; value: string }> = [];

  sortByOptions: Array<{ label: string; value: 'price' | 'created_at' | 'updated_at' | 'name' }> = [];

  get brandOptions(): { label: string; value: number }[] {
    const allowed = this.filterOptionsStore.options()?.brand_ids ?? null;
    const selected = this.filterBrandId;
    return this.brandStore.brands()
      .filter(b => allowed === null || allowed.includes(b.id) || b.id === selected)
      .map(b => ({ label: b.name, value: b.id }));
  }

  get categoryTreeNodes(): TreeNode[] {
    const all = this.categoryStore.categories();
    if (!all.length) return [];

    const byId = new Map(all.map(c => [c.id, c] as const));
    const parentIds = new Set(
      all.map(c => c.parent_id).filter((id): id is number => id !== null),
    );
    const allowed = this.filterOptionsStore.options()?.category_ids ?? null;
    const selected = this.filterCategoryId;

    const selectableLeaves = all
      .filter(c => !parentIds.has(c.id))
      .filter(c => allowed === null || allowed.includes(c.id) || c.id === selected);
    const visibleIds = new Set<number>();
    for (const leaf of selectableLeaves) {
      let current: CategoryFullResponse | undefined = leaf;
      while (current) {
        visibleIds.add(current.id);
        current = current.parent_id ? byId.get(current.parent_id) : undefined;
      }
    }

    type TypeGroup = Map<string, CategoryFullResponse[]>;
    type GenderGroup = Map<string, TypeGroup>;
    const root: GenderGroup = new Map();

    for (const category of all.filter(c => visibleIds.has(c.id))) {
      const types = root.get(category.gender) ?? new Map();
      const bucket = types.get(category.product_type) ?? [];
      bucket.push(category);
      types.set(category.product_type, bucket);
      root.set(category.gender, types);
    }

    const cmp = (a: string, b: string) => a.localeCompare(b);
    const buildCategoryNodes = (categories: CategoryFullResponse[], typeKey: string): TreeNode[] => {
      const groupIds = new Set(categories.map(c => c.id));
      const childrenByParent = new Map<number | null, CategoryFullResponse[]>();
      for (const category of categories) {
        const parentId = category.parent_id && groupIds.has(category.parent_id)
          ? category.parent_id
          : null;
        const bucket = childrenByParent.get(parentId) ?? [];
        bucket.push(category);
        childrenByParent.set(parentId, bucket);
      }

      const buildNode = (category: CategoryFullResponse): TreeNode => {
        const children = (childrenByParent.get(category.id) ?? [])
          .sort((a, b) => cmp(a.name, b.name))
          .map(buildNode);
        const isLeaf = children.length === 0 && !parentIds.has(category.id);
        return this.upsertCategoryNode(`c:${category.id}`, {
          label: category.name,
          data: isLeaf ? { id: category.id } : undefined,
          selectable: isLeaf,
          icon: isLeaf ? 'pi pi-tag' : 'pi pi-folder',
          children,
        });
      };

      const buildRootNodes = (category: CategoryFullResponse): TreeNode[] => {
        const children = childrenByParent.get(category.id) ?? [];
        if (
          category.id !== selected &&
          children.length > 0 &&
          this.isRedundantTypeRoot(category, typeKey)
        ) {
          return [...children].sort((a, b) => cmp(a.name, b.name)).map(buildNode);
        }

        return [buildNode(category)];
      };

      return (childrenByParent.get(null) ?? [])
        .sort((a, b) => cmp(a.name, b.name))
        .flatMap(buildRootNodes);
    };

    const nodes: TreeNode[] = [];
    const sortedGenders = [...root.entries()].sort((a, b) =>
      cmp(this.t(`genders.${a[0]}`, a[0]), this.t(`genders.${b[0]}`, b[0])),
    );
    for (const [genderKey, types] of sortedGenders) {
      const genderNode = this.upsertCategoryNode(`g:${genderKey}`, {
        label: this.t(`genders.${genderKey}`, this.toTitleCase(genderKey)),
        icon: 'pi pi-users',
        selectable: false,
        children: [],
      });
      const sortedTypes = [...types.entries()].sort((a, b) =>
        cmp(this.t(`productTypes.${a[0]}`, a[0]), this.t(`productTypes.${b[0]}`, b[0])),
      );
      for (const [typeKey, categories] of sortedTypes) {
        const typeNode = this.upsertCategoryNode(`g:${genderKey}/t:${typeKey}`, {
          label: this.t(`productTypes.${typeKey}`, this.toTitleCase(typeKey)),
          icon: 'pi pi-folder',
          selectable: false,
          children: buildCategoryNodes(categories, typeKey),
        });
        genderNode.children!.push(typeNode);
      }
      nodes.push(genderNode);
    }
    return nodes;
  }

  get conditionFilterOptions(): { label: string; value: string }[] {
    const allowed = this.filterOptionsStore.options()?.conditions ?? null;
    const selected = this.filterCondition;
    return this.conditionOptions.filter(
      o => allowed === null || allowed.includes(o.value) || o.value === selected,
    );
  }

  get colorOptions(): { label: string; value: string }[] {
    const colors = this.filterOptionsStore.options()?.colors ?? [];
    const selected = this.filterColor;
    const merged = selected && !colors.includes(selected) ? [...colors, selected] : colors;
    return merged.map(c => ({ label: this.t(`colors.${c}`, this.toTitleCase(c)), value: c }));
  }

  get currentSizeGroup(): string | null {
    const sizes = this.filterOptionsStore.sizes();
    return sizes?.size_group ?? this.selectedCategory?.size_group ?? null;
  }

  get sizeValueOptions(): { label: string; value: string }[] {
    const sizes = this.filterOptionsStore.sizes();
    if (!sizes) return [];
    const selected = this.filterSizeValues;
    const values = sizes.values ?? [];
    const merged = this.mergeSelectedOptions(values, selected);
    return merged.map(v => ({ label: v, value: v }));
  }

  get sizeValue2Options(): { label: string; value: string }[] {
    const sizes = this.filterOptionsStore.sizes();
    if (!sizes) return [];
    const values = sizes.values2 ?? [];
    const merged = this.mergeSelectedOptions(values, this.filterSizeValues2);
    return merged.map(v => ({ label: v, value: v }));
  }

  get sizeSystemOptions(): { label: string; value: string }[] {
    const sizes = this.filterOptionsStore.sizes();
    const systems = sizes?.systems ?? [];
    const merged = this.mergeSelectedOptions(systems, this.filterSizeSystems);
    return merged.map(v => ({ label: this.t(`sizeSystems.${v}`, v), value: v }));
  }

  get shoeWidthOptions(): { label: string; value: string }[] {
    const sizes = this.filterOptionsStore.sizes();
    const widths = sizes?.widths ?? [];
    const merged = this.mergeSelectedOptions(widths, this.filterShoeWidths);
    return merged.map(v => ({
      label: this.t(`admin.products.form.options.shoeWidths.${v}`, this.toTitleCase(v)),
      value: v,
    }));
  }

  get showSizeSystemFilter(): boolean {
    return ['shoe', 'ring', 'letter_or_numeric'].includes(this.currentSizeGroup ?? '') &&
      this.sizeSystemOptions.length > 0;
  }

  get showShoeWidthFilter(): boolean {
    return this.currentSizeGroup === 'shoe' && this.shoeWidthOptions.length > 0;
  }

  get sizeValuePlaceholder(): string {
    switch (this.currentSizeGroup) {
      case 'shoe':
        return this.t('admin.products.list.allShoeSizes');
      case 'waist_length':
        return this.t('admin.products.list.allWaists');
      case 'ring':
        return this.t('admin.products.list.allRingSizes');
      default:
        return this.t('admin.products.list.allSizes');
    }
  }

  get sizeValue2Placeholder(): string {
    return this.t('admin.products.list.allLengths');
  }

  get selectedCategory(): CategoryFullResponse | null {
    if (this.filterCategoryId === null) return null;
    return this.categoryStore.categories().find(c => c.id === this.filterCategoryId) ?? null;
  }

  get selectedCategoryPath(): string | null {
    const category = this.selectedCategory;
    if (!category) return null;

    const byId = new Map(this.categoryStore.categories().map(c => [c.id, c] as const));
    const path: string[] = [];
    let current: CategoryFullResponse | undefined = category;
    while (current) {
      path.unshift(current.name);
      current = current.parent_id ? byId.get(current.parent_id) : undefined;
    }

    const displayPath = this.pathWithoutRedundantTypeRoot(path, category.product_type);

    return [
      this.t(`genders.${category.gender}`, this.toTitleCase(category.gender)),
      this.t(`productTypes.${category.product_type}`, this.toTitleCase(category.product_type)),
      ...displayPath,
    ].join(' / ');
  }

  get genderLockedFromCategory(): boolean {
    return this.selectedCategory !== null;
  }

  get sizeFieldState(): 'no-category' | 'loading' | 'one-size' | 'no-products' | 'ready' {
    if (this.filterCategoryId === null) return 'no-category';
    const sizes = this.filterOptionsStore.sizes();
    if (!sizes && this.filterOptionsStore.loadingSizes()) return 'loading';
    if (!sizes) return 'no-products';
    if (['one_size', 'dimensions'].includes(sizes.size_group)) return 'one-size';
    if ((sizes.values?.length ?? 0) === 0 && (sizes.values2?.length ?? 0) === 0) return 'no-products';
    return 'ready';
  }

  get sizeFilterDisabled(): boolean {
    return this.sizeFieldState !== 'ready';
  }

  get sizeFieldPlaceholder(): string {
    switch (this.sizeFieldState) {
      case 'no-category':
        return this.t('admin.products.list.selectCategoryFirst');
      case 'loading':
        return this.t('admin.products.list.loadingSizes');
      case 'one-size':
        return this.t('admin.products.list.oneSize');
      case 'no-products':
        return this.t('admin.products.list.noSizesAvailable');
      default:
        return this.t('admin.products.list.allSizes');
    }
  }

  get pricePlaceholderMin(): string {
    return this.filterOptionsStore.options()?.price_min ?? '';
  }

  get pricePlaceholderMax(): string {
    return this.filterOptionsStore.options()?.price_max ?? '';
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
    if (this.priceDebounceTimer) {
      clearTimeout(this.priceDebounceTimer);
    }
    if (this.sizeDebounceTimer) {
      clearTimeout(this.sizeDebounceTimer);
    }
    this.langChangeSub?.unsubscribe();
  }

  onFilterChange(): void {
    this.applyFilters();
  }

  onCategoryNodeChange(node: TreeNode | null): void {
    const id = (node?.data as { id: number } | undefined)?.id ?? null;
    this.filterCategoryId = id;
    this.selectedCategoryNode = node ?? null;
    this.onCategoryChange();
  }

  onCategoryChange(): void {
    const cat = this.selectedCategory;
    this.filterGender = cat ? cat.gender : null;
    this.resetSizeFilters();
    this.filterOptionsStore.clearSizes();
    this.applyFilters();
  }

  onSizeFilterChange(): void {
    this.applyFilters();
  }

  onSizeValueFilterChange(): void {
    if (this.sizeDebounceTimer) {
      clearTimeout(this.sizeDebounceTimer);
    }
    this.sizeDebounceTimer = setTimeout(() => {
      this.applyProductFilters();
    }, 120);
  }

  toggleFiltersPanel(): void {
    const nextCollapsed = !this.filtersCollapsed;
    const nextOptions = this.pageSizeOptionsFor(nextCollapsed);
    const currentPerPage = this.store.perPage();
    this.filtersCollapsed = nextCollapsed;

    if (!nextOptions.includes(currentPerPage)) {
      const nextPerPage = this.defaultPerPage(nextCollapsed);
      void this.store.setPage(1, nextPerPage);
      this.syncQueryParams(1, nextPerPage);
    }
  }

  onPriceInput(): void {
    if (this.priceDebounceTimer) {
      clearTimeout(this.priceDebounceTimer);
    }
    this.priceDebounceTimer = setTimeout(() => {
      this.applyFilters();
    }, 400);
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

  get filterPanelToggleLabel(): string {
    return this.filtersCollapsed
      ? this.t('admin.products.list.showFilters')
      : this.t('admin.products.list.hideFilters');
  }

  get filterPanelToggleIcon(): string {
    return this.filtersCollapsed ? 'pi pi-filter' : 'pi pi-angle-left';
  }

  get pageSizeOptions(): number[] {
    return this.pageSizeOptionsFor(this.filtersCollapsed);
  }

  private buildLocalizedOptions(): void {
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

  private defaultPerPage(collapsed = this.filtersCollapsed): number {
    return collapsed ? 18 : 20;
  }

  private pageSizeOptionsFor(collapsed: boolean): number[] {
    return collapsed ? this.pageSizeOptionsCollapsed : this.pageSizeOptionsOpen;
  }

  private normalizePerPage(value: number): number {
    return Number.isFinite(value) && this.pageSizeOptions.includes(value)
      ? value
      : this.defaultPerPage();
  }

  private async initializeFromQuery(): Promise<void> {
    await this.categoryStore.loadCategories();
    const { page, perPage } = this.applyStateFromQuery();
    this.syncSelectedCategoryNode();

    if (perPage !== this.store.perPage()) {
      await this.store.setPage(1, perPage);
    }

    await this.store.setFilters(this.buildFilterPayload());
    void this.refreshDependentOptions();

    if (page > 1) {
      await this.store.setPage(page, perPage);
    }
  }

  private syncSelectedCategoryNode(): void {
    if (this.filterCategoryId === null) {
      this.selectedCategoryNode = null;
      return;
    }
    this.selectedCategoryNode = this.findCategoryNode(
      this.categoryTreeNodes,
      this.filterCategoryId,
    );
  }

  private findCategoryNode(nodes: TreeNode[], id: number): TreeNode | null {
    for (const n of nodes) {
      if ((n.data as { id?: number } | undefined)?.id === id) return n;
      if (n.children) {
        const found = this.findCategoryNode(n.children, id);
        if (found) return found;
      }
    }
    return null;
  }

  private upsertCategoryNode(key: string, next: Omit<TreeNode, 'key'>): TreeNode {
    const existing = this.categoryNodeCache.get(key);
    if (!existing) {
      const node = { key, ...next };
      this.categoryNodeCache.set(key, node);
      return node;
    }

    const expanded = existing.expanded;
    Object.assign(existing, next);
    existing.key = key;
    if (expanded !== undefined) {
      existing.expanded = expanded;
    }
    return existing;
  }

  private applyFilters(): void {
    this.applyProductFilters();
    void this.refreshDependentOptions();
  }

  private applyProductFilters(): void {
    void this.store.setFilters(this.buildFilterPayload());
    this.syncQueryParams(1);
  }

  private buildFilterPayload() {
    return {
      search: this.filterSearch.trim() || undefined,
      brand_id: this.filterBrandId ?? undefined,
      category_id: this.filterCategoryId ?? undefined,
      status: this.filterStatus ?? undefined,
      condition: this.filterCondition ?? undefined,
      gender: this.filterGender ?? undefined,
      color: this.filterColor ?? undefined,
      price_min: this.filterPriceMin?.trim() || undefined,
      price_max: this.filterPriceMax?.trim() || undefined,
      size_values: this.toCsv(this.filterSizeValues),
      size_values2: this.toCsv(this.filterSizeValues2),
      size_systems: this.toCsv(this.filterSizeSystems),
      shoe_widths: this.toCsv(this.filterShoeWidths),
      sort_by: this.sortBy,
      sort_order: this.sortOrder,
    };
  }

  private refreshDependentOptions(): Promise<void[]> {
    const optionsScope = {
      brand_id: this.filterBrandId ?? undefined,
      category_id: this.filterCategoryId ?? undefined,
      gender: this.filterGender ?? undefined,
      status: this.filterStatus ?? undefined,
      condition: this.filterCondition ?? undefined,
      color: this.filterColor ?? undefined,
      price_min: this.filterPriceMin?.trim() || undefined,
      price_max: this.filterPriceMax?.trim() || undefined,
    };
    const optionsPromise = this.filterOptionsStore.loadOptions(optionsScope);
    const sizesPromise = this.filterCategoryId
      ? this.filterOptionsStore.loadSizes({
          category_id: this.filterCategoryId,
          brand_id: this.filterBrandId ?? undefined,
          gender: this.filterGender ?? undefined,
          status: this.filterStatus ?? undefined,
          condition: this.filterCondition ?? undefined,
          color: this.filterColor ?? undefined,
          size_systems: this.toCsv(this.filterSizeSystems),
          price_min: this.filterPriceMin?.trim() || undefined,
          price_max: this.filterPriceMax?.trim() || undefined,
        })
      : Promise.resolve(this.filterOptionsStore.clearSizes());
    return Promise.all([optionsPromise, sizesPromise]);
  }

  clearFilters(): void {
    this.filterSearch = '';
    this.filterBrandId = null;
    this.filterCategoryId = null;
    this.selectedCategoryNode = null;
    this.filterStatus = null;
    this.filterCondition = null;
    this.filterGender = null;
    this.filterColor = null;
    this.resetSizeFilters();
    this.filterPriceMin = null;
    this.filterPriceMax = null;
    this.sortBy = 'created_at';
    this.sortOrder = 'desc';
    void this.store.setFilters({
      sort_by: this.sortBy,
      sort_order: this.sortOrder,
    });
    void this.refreshDependentOptions();
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
      this.filterCategoryId !== null,
      this.filterStatus !== null,
      this.filterCondition !== null,
      this.filterGender !== null && !this.genderLockedFromCategory,
      this.filterColor !== null,
      this.filterSizeValues.length > 0,
      this.filterSizeValues2.length > 0,
      this.filterSizeSystems.length > 0,
      this.filterShoeWidths.length > 0,
      (this.filterPriceMin?.trim().length ?? 0) > 0,
      (this.filterPriceMax?.trim().length ?? 0) > 0,
      this.sortBy !== 'created_at',
      this.sortOrder !== 'desc',
    ].filter(Boolean).length;
  }

  private applyStateFromQuery(): { page: number; perPage: number } {
    const params = this.route.snapshot.queryParamMap;
    const search = params.get('search');
    const brandId = params.get('brand_id');
    const categoryId = params.get('category_id');
    const page = params.get('page');
    const perPage = params.get('per_page');
    const sortBy = params.get('sort_by');
    const sortOrder = params.get('sort_order');
    const view = params.get('view');

    this.filterSearch = search ?? '';
    this.filterBrandId = brandId ? Number(brandId) : null;
    this.filterCategoryId = categoryId ? Number(categoryId) : null;
    this.filterStatus = params.get('status');
    this.filterCondition = params.get('condition');
    this.filterGender = params.get('gender');
    this.filterColor = params.get('color');
    this.filterSizeValues = this.fromCsv(params.get('size_values') ?? params.get('size_value'));
    this.filterSizeValues2 = this.fromCsv(params.get('size_values2') ?? params.get('size_value2'));
    this.filterSizeSystems = this.fromCsv(params.get('size_systems') ?? params.get('size_system'));
    this.filterShoeWidths = this.fromCsv(params.get('shoe_widths'));
    this.filterPriceMin = params.get('price_min');
    this.filterPriceMax = params.get('price_max');
    this.sortBy = this.isSortField(sortBy) ? sortBy : 'created_at';
    this.sortOrder = sortOrder === 'asc' ? 'asc' : 'desc';
    this.viewMode = view === 'table' ? 'table' : 'cards';

    const pageNum = page ? Number(page) : 1;
    const fallbackPerPage = this.defaultPerPage();
    const perPageNum = perPage ? Number(perPage) : fallbackPerPage;
    const normalizedPage = Number.isFinite(pageNum) && pageNum > 0 ? pageNum : 1;
    const normalizedPerPage = this.normalizePerPage(perPageNum);

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
        category_id: this.filterCategoryId ?? null,
        status: this.filterStatus ?? null,
        condition: this.filterCondition ?? null,
        gender: this.filterGender ?? null,
        color: this.filterColor ?? null,
        size_value: null,
        size_value2: null,
        size_values: this.toCsv(this.filterSizeValues) ?? null,
        size_values2: this.toCsv(this.filterSizeValues2) ?? null,
        size_system: null,
        size_systems: this.toCsv(this.filterSizeSystems) ?? null,
        shoe_widths: this.toCsv(this.filterShoeWidths) ?? null,
        price_min: this.filterPriceMin?.trim() || null,
        price_max: this.filterPriceMax?.trim() || null,
        sort_by: this.sortBy !== 'created_at' ? this.sortBy : null,
        sort_order: this.sortOrder !== 'desc' ? this.sortOrder : null,
        page: currentPage > 1 ? currentPage : null,
        per_page: currentPerPage !== this.defaultPerPage() ? currentPerPage : null,
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

  private toTitleCase(value: string): string {
    return value.length ? value[0].toUpperCase() + value.slice(1) : value;
  }

  private isRedundantTypeRoot(category: CategoryFullResponse, typeKey: string): boolean {
    return (
      category.parent_id === null &&
      category.product_type === typeKey &&
      this.normalizeCategoryToken(category.name) === this.normalizeCategoryToken(typeKey)
    );
  }

  private pathWithoutRedundantTypeRoot(path: string[], typeKey: string): string[] {
    return path.length > 0 &&
      this.normalizeCategoryToken(path[0]) === this.normalizeCategoryToken(typeKey)
      ? path.slice(1)
      : path;
  }

  private normalizeCategoryToken(value: string): string {
    return value.toLowerCase().replace(/[^a-z0-9]+/g, '');
  }

  private resetSizeFilters(): void {
    this.filterSizeValues = [];
    this.filterSizeValues2 = [];
    this.filterSizeSystems = [];
    this.filterShoeWidths = [];
  }

  private mergeSelectedOptions(values: string[], selected: string[]): string[] {
    return [
      ...values,
      ...selected.filter(value => !values.includes(value)),
    ];
  }

  private toCsv(values: string[]): string | undefined {
    return values.length > 0 ? values.join(',') : undefined;
  }

  private fromCsv(value: string | null): string[] {
    if (!value) return [];
    return value
      .split(',')
      .map(v => v.trim())
      .filter(Boolean);
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
