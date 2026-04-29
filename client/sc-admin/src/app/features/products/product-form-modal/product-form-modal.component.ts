import {
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  OnDestroy,
  effect,
  inject,
  input,
  output,
  signal,
} from '@angular/core';
import { FormBuilder, FormGroup, FormsModule, ReactiveFormsModule, Validators } from '@angular/forms';
import { Router } from '@angular/router';
import { DialogModule } from 'primeng/dialog';
import { ButtonModule } from 'primeng/button';
import { InputTextModule } from 'primeng/inputtext';
import { InputNumberModule } from 'primeng/inputnumber';
import { SelectModule } from 'primeng/select';
import { TreeSelectModule } from 'primeng/treeselect';
import { MultiSelectModule } from 'primeng/multiselect';
import { CheckboxModule } from 'primeng/checkbox';
import { TextareaModule } from 'primeng/textarea';
import { TagModule } from 'primeng/tag';
import { ChipModule } from 'primeng/chip';
import { DividerModule } from 'primeng/divider';
import { ToastModule } from 'primeng/toast';
import { TooltipModule } from 'primeng/tooltip';
import { MessageService, TreeNode } from 'primeng/api';
import { TranslateModule, TranslateService } from '@ngx-translate/core';
import { Subscription, firstValueFrom } from 'rxjs';
import { ProductApiService } from '../../../core/services/product-api.service';
import { BrandApiService } from '../../../core/services/brand-api.service';
import { CategoryApiService } from '../../../core/services/category-api.service';
import { TagApiService } from '../../../core/services/tag-api.service';
import { PurchaseLocationApiService } from '../../../core/services/purchase-location-api.service';
import { Brand } from '../../../core/models/brand.model';
import { CategoryFullResponse } from '../../../core/models/category.model';
import { StyleTag, VibeTag, Season } from '../../../core/models/tag.model';
import { PurchaseLocation } from '../../../core/models/purchase-location.model';
import { AdminProductDTO, CreateProductRequest, UpdateProductRequest } from '../../../core/models/product.model';
import { BrandFormModalComponent } from '../../brands/brand-form-modal/brand-form-modal.component';
import {
  isRedundantTypeRoot,
  pathWithoutRedundantTypeRoot,
} from '../category-tree.utils';

@Component({
  selector: 'app-product-form-modal',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [
    FormsModule,
    ReactiveFormsModule,
    DialogModule,
    ButtonModule,
    InputTextModule,
    InputNumberModule,
    SelectModule,
    TreeSelectModule,
    MultiSelectModule,
    CheckboxModule,
    TextareaModule,
    TagModule,
    ChipModule,
    DividerModule,
    ToastModule,
    TooltipModule,
    TranslateModule,
    BrandFormModalComponent,
  ],
  providers: [MessageService],
  templateUrl: './product-form-modal.component.html',
  styleUrl: './product-form-modal.component.scss',
})
export class ProductFormModalComponent implements OnDestroy {
  readonly visible = input.required<boolean>();
  readonly productId = input<string | null>(null);
  readonly visibleChange = output<boolean>();
  readonly saved = output<void>();

  private readonly fb = inject(FormBuilder);
  private readonly productApi = inject(ProductApiService);
  private readonly brandApi = inject(BrandApiService);
  private readonly categoryApi = inject(CategoryApiService);
  private readonly tagApi = inject(TagApiService);
  private readonly purchaseLocationApi = inject(PurchaseLocationApiService);
  private readonly messageService = inject(MessageService);
  private readonly cdr = inject(ChangeDetectorRef);
  private readonly translate = inject(TranslateService);
  private readonly router = inject(Router);

  form!: FormGroup;
  existingProduct: AdminProductDTO | null = null;
  brands: Brand[] = [];
  categories: CategoryFullResponse[] = [];
  styleTags: StyleTag[] = [];
  vibeTags: VibeTag[] = [];
  seasons: Season[] = [];
  purchaseLocations: PurchaseLocation[] = [];
  selectedCategoryNode: TreeNode | null = null;

  protected readonly loadingProduct = signal(false);
  protected readonly saving = signal(false);
  protected readonly staleVersionError = signal(false);
  protected readonly selectedProductType = signal<string | null>(null);
  protected readonly selectedSizeGroup = signal<string | null>(null);
  protected readonly brandModalVisible = signal(false);
  private langChangeSub: Subscription | null = null;

  statuses: Array<{ label: string; value: string }> = [];

  conditionOptions: Array<{ label: string; value: string }> = [];

  currencies = ['RSD', 'EUR', 'USD', 'GBP'];

  fitOptions: Array<{ label: string; value: string }> = [];

  sizeSystems: Array<{ label: string; value: string }> = [];

  shoeWidths: Array<{ label: string; value: string }> = [];

  handleTypes: Array<{ label: string; value: string }> = [];
  private readonly categoryNodeCache = new Map<string, TreeNode>();

  private readonly statusValues = [
    'intake',
    'inspection',
    'rejected',
    'preparation',
    'photo_queue',
    'photo_done',
    'ready',
    'reserved',
    'sold',
    'returned',
  ] as const;

  private readonly conditionValues = [
    'new_with_tags',
    'excellent',
    'good',
    'fair',
  ] as const;

  private readonly fitValues = ['regular', 'slim', 'oversized', 'relaxed'] as const;
  private readonly sizeSystemValues = ['EU', 'US', 'UK'] as const;
  private readonly shoeWidthValues = ['narrow', 'regular', 'wide'] as const;
  private readonly handleTypeValues = ['shoulder', 'crossbody', 'hand', 'backpack', 'tote'] as const;

  get categoryTreeNodes(): TreeNode[] {
    if (!this.categories.length) {
      return [];
    }

    const byId = new Map(this.categories.map(cat => [cat.id, cat] as const));
    const parentIds = new Set(
      this.categories
        .map(cat => cat.parent_id)
        .filter((parentId): parentId is number => parentId !== null),
    );

    const selectedCategoryId = this.form?.get('category_id')?.value as number | null | undefined;
    const selectableCategories = this.categories.filter(
      cat => !parentIds.has(cat.id) || cat.id === selectedCategoryId,
    );

    const visibleIds = new Set<number>();
    for (const category of selectableCategories) {
      let current: CategoryFullResponse | undefined = category;
      while (current) {
        visibleIds.add(current.id);
        current = current.parent_id ? byId.get(current.parent_id) : undefined;
      }
    }

    type TypeGroup = Map<string, CategoryFullResponse[]>;
    type GenderGroup = Map<string, TypeGroup>;
    const root: GenderGroup = new Map();

    for (const category of this.categories.filter(cat => visibleIds.has(cat.id))) {
      const types = root.get(category.gender) ?? new Map();
      const bucket = types.get(category.product_type) ?? [];
      bucket.push(category);
      types.set(category.product_type, bucket);
      root.set(category.gender, types);
    }

    const cmp = (a: string, b: string) => a.localeCompare(b);
    const buildCategoryNodes = (categories: CategoryFullResponse[], typeKey: string): TreeNode[] => {
      const groupIds = new Set(categories.map(cat => cat.id));
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
        const isSelected = category.id === selectedCategoryId;
        const isLeaf = children.length === 0 && !parentIds.has(category.id);
        const selectable = isLeaf || isSelected;

        return this.upsertCategoryNode(`c:${category.id}`, {
          label: category.name,
          data: selectable ? { id: category.id } : undefined,
          selectable,
          icon: isLeaf ? 'pi pi-tag' : 'pi pi-folder',
          children,
        });
      };

      const buildRootNodes = (category: CategoryFullResponse): TreeNode[] => {
        const children = childrenByParent.get(category.id) ?? [];
        if (
          category.id !== selectedCategoryId &&
          children.length > 0 &&
          isRedundantTypeRoot(category, typeKey)
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

  get selectedCategoryPath(): string | null {
    const catId = this.form?.get('category_id')?.value as number | null | undefined;
    if (!catId) return null;

    const category = this.categories.find(cat => cat.id === catId);
    if (!category) return null;

    const byId = new Map(this.categories.map(cat => [cat.id, cat] as const));
    const path: string[] = [];
    let current: CategoryFullResponse | undefined = category;
    while (current) {
      path.unshift(current.name);
      current = current.parent_id ? byId.get(current.parent_id) : undefined;
    }

    const displayPath = pathWithoutRedundantTypeRoot(path, category.product_type);

    return [
      this.t(`genders.${category.gender}`, this.toTitleCase(category.gender)),
      this.t(`productTypes.${category.product_type}`, this.toTitleCase(category.product_type)),
      ...displayPath,
    ].join(' / ');
  }

  private toTitleCase(value: string): string {
    return value.length ? value[0].toUpperCase() + value.slice(1) : value;
  }

  constructor() {
    this.buildForm();
    this.buildLocalizedOptions();
    this.langChangeSub = this.translate.onLangChange.subscribe(() => {
      this.buildLocalizedOptions();
      this.cdr.markForCheck();
    });

    effect(() => {
      const isVisible = this.visible();
      if (isVisible) {
        this.staleVersionError.set(false);
        this.selectedProductType.set(null);
        this.selectedSizeGroup.set(null);
        this.selectedCategoryNode = null;
        this.existingProduct = null;
        this.buildForm();
        this.loadDropdownData();

        const pid = this.productId();
        if (pid) {
          this.loadProduct(pid);
        }
      }
    });
  }

  ngOnDestroy(): void {
    this.langChangeSub?.unsubscribe();
  }

  private buildLocalizedOptions(): void {
    this.statuses = this.statusValues.map(value => ({
      label: this.t(`statuses.${value}`, value),
      value,
    }));
    this.conditionOptions = this.conditionValues.map(value => ({
      label: this.t(`conditions.${value}`, value),
      value,
    }));
    this.fitOptions = this.fitValues.map(value => ({
      label: this.t(`admin.products.form.options.fit.${value}`, value),
      value,
    }));
    this.sizeSystems = this.sizeSystemValues.map(value => ({
      label: this.t(`sizeSystems.${value}`, value),
      value,
    }));
    this.shoeWidths = this.shoeWidthValues.map(value => ({
      label: this.t(`admin.products.form.options.shoeWidths.${value}`, value),
      value,
    }));
    this.handleTypes = this.handleTypeValues.map(value => ({
      label: this.t(`handleTypes.${value}`, value),
      value,
    }));
  }

  private buildForm(): void {
    this.form = this.fb.group({
      name: ['', Validators.required],
      brand_id: [null, Validators.required],
      category_id: [null, Validators.required],
      status: ['intake'],
      purchase_price: [''],
      currency: ['RSD'],
      purchase_location_id: [null],
      ai_notes: [''],
      details: this.fb.group({
        condition: ['', Validators.required],
        material: [''],
        color: [''],
        year_of_release: [null],
        is_vintage: [false],
        is_collab: [false],
        collab_name: [''],
        is_limited_edition: [false],
        special_notes: [''],
      }),
      size: this.fb.group({
        size_value: [''],
        size_value2: [''],
        size_system: [null],
        measurement_cm: [''],
      }),
      type_details: this.fb.group({
        // clothing
        fit: [null],
        // footwear
        shoe_width: [null],
        insole_length_cm: [''],
        // bags
        width_cm: [''],
        height_cm: [''],
        depth_cm: [''],
        handle_type: [null],
        bag_size_label: [''],
        // jewelry
        metal: [''],
        stone: [''],
        clasp_type: [''],
      }),
      style_tag_ids: [[]],
      vibe_tag_ids: [[]],
      season_ids: [[]],
    });
  }

  onCategoryNodeChange(node: TreeNode | null): void {
    const id = (node?.data as { id: number } | undefined)?.id ?? null;
    this.selectedCategoryNode = node ?? null;
    this.form.get('category_id')?.setValue(id);
    this.form.get('category_id')?.markAsTouched();
    this.onCategoryChange();
  }

  onCategoryChange(resetDependentFields = true): void {
    const catId = this.form.get('category_id')?.value;
    const cat = this.categories.find(c => c.id === catId);
    this.selectedProductType.set(cat?.product_type ?? null);
    this.selectedSizeGroup.set(cat?.size_group ?? null);
    this.syncSelectedCategoryNode();
    if (resetDependentFields) {
      this.resetCategoryDependentFields();
    }
    this.cdr.markForCheck();
  }

  private resetCategoryDependentFields(): void {
    this.form.get('size')?.reset({
      size_value: '',
      size_value2: '',
      size_system: null,
      measurement_cm: '',
    });
    this.form.get('type_details')?.reset({
      fit: null,
      shoe_width: null,
      insole_length_cm: '',
      width_cm: '',
      height_cm: '',
      depth_cm: '',
      handle_type: null,
      bag_size_label: '',
      metal: '',
      stone: '',
      clasp_type: '',
    });
  }

  private syncSelectedCategoryNode(): void {
    const catId = this.form?.get('category_id')?.value as number | null | undefined;
    if (!catId) {
      this.selectedCategoryNode = null;
      return;
    }

    this.selectedCategoryNode = this.findCategoryNode(this.categoryTreeNodes, catId);
  }

  private findCategoryNode(nodes: TreeNode[], id: number): TreeNode | null {
    for (const node of nodes) {
      if ((node.data as { id?: number } | undefined)?.id === id) {
        return node;
      }
      if (node.children) {
        const found = this.findCategoryNode(node.children, id);
        if (found) {
          return found;
        }
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

  private async loadDropdownData(): Promise<void> {
    try {
      const [brandsRes, categoriesRes, stylesRes, vibesRes, seasonsRes, purchaseLocationsRes] = await Promise.all([
        firstValueFrom(this.brandApi.getAll()),
        firstValueFrom(this.categoryApi.getAll()),
        firstValueFrom(this.tagApi.getStyles()),
        firstValueFrom(this.tagApi.getVibes()),
        firstValueFrom(this.tagApi.getSeasons()),
        firstValueFrom(this.purchaseLocationApi.getAll()),
      ]);
      if (brandsRes.success && brandsRes.data) this.brands = brandsRes.data;
      if (categoriesRes.success && categoriesRes.data) this.categories = categoriesRes.data;
      if (stylesRes.success && stylesRes.data) this.styleTags = stylesRes.data;
      if (vibesRes.success && vibesRes.data) this.vibeTags = vibesRes.data;
      if (seasonsRes.success && seasonsRes.data) this.seasons = seasonsRes.data;
      if (purchaseLocationsRes.success && purchaseLocationsRes.data) {
        this.purchaseLocations = purchaseLocationsRes.data;
      }
      this.syncDerivedSelectionsFromProduct();
      this.syncSelectedCategoryNode();
      this.cdr.markForCheck();
    } catch {
      // Silently fail -- user will see empty dropdowns
    }
  }

  private async loadProduct(id: string): Promise<void> {
    this.loadingProduct.set(true);
    try {
      const res = await firstValueFrom(this.productApi.getProduct(id));
      if (res.success && res.data) {
        this.existingProduct = res.data;
        this.patchFormFromProduct(res.data);
        this.cdr.markForCheck();
      }
    } catch {
      this.messageService.add({
        severity: 'error',
        summary: this.t('admin.products.form.messages.loadFailedSummary', this.t('common.error')),
        detail: this.t('admin.products.form.messages.loadFailedDetail', 'Failed to load product'),
      });
    } finally {
      this.loadingProduct.set(false);
    }
  }

  private patchFormFromProduct(p: AdminProductDTO): void {
    this.selectedProductType.set(p.product_type);
    this.selectedSizeGroup.set(p.category.size_group);

    this.form.patchValue({
      name: p.name,
      brand_id: p.brand_id,
      category_id: p.category_id,
      status: p.status,
      purchase_price: p.purchase_price ?? '',
      currency: p.currency,
      purchase_location_id: null,
      ai_notes: p.ai_notes ?? '',
      details: {
        condition: p.details.condition,
        material: p.details.material ?? '',
        color: p.details.color ?? '',
        year_of_release: p.details.year_of_release ?? null,
        is_vintage: p.details.is_vintage,
        is_collab: p.details.is_collab,
        collab_name: p.details.collab_name ?? '',
        is_limited_edition: p.details.is_limited_edition,
        special_notes: p.details.special_notes ?? '',
      },
      size: {
        size_value: p.size?.size_value ?? '',
        size_value2: p.size?.size_value2 ?? '',
        size_system: p.size?.size_system ?? null,
        measurement_cm: p.size?.measurement_cm ?? '',
      },
      style_tag_ids: this.mapTagNamesToIds(p.tags.styles, this.styleTags),
      vibe_tag_ids: this.mapTagNamesToIds(p.tags.vibes, this.vibeTags),
      season_ids: this.mapTagNamesToIds(p.tags.seasons, this.seasons),
    });

    // Patch type_details based on product_type
    if (p.type_details) {
      const td = p.type_details as any;
      this.form.get('type_details')?.patchValue(td);
    }
    this.syncDerivedSelectionsFromProduct();
    this.syncSelectedCategoryNode();
  }

  private mapTagNamesToIds(names: string[], options: Array<{ id: number; name: string }>): number[] {
    if (!names.length || !options.length) {
      return [];
    }
    const wanted = new Set(names.map(name => name.toLowerCase()));
    return options.filter(tag => wanted.has(tag.name.toLowerCase())).map(tag => tag.id);
  }

  private syncDerivedSelectionsFromProduct(): void {
    if (!this.existingProduct) {
      return;
    }

    const patch: Record<string, unknown> = {};
    const styleTagIds = this.mapTagNamesToIds(this.existingProduct.tags.styles, this.styleTags);
    const vibeTagIds = this.mapTagNamesToIds(this.existingProduct.tags.vibes, this.vibeTags);
    const seasonIds = this.mapTagNamesToIds(this.existingProduct.tags.seasons, this.seasons);

    if (styleTagIds.length || this.existingProduct.tags.styles.length === 0) {
      patch['style_tag_ids'] = styleTagIds;
    }
    if (vibeTagIds.length || this.existingProduct.tags.vibes.length === 0) {
      patch['vibe_tag_ids'] = vibeTagIds;
    }
    if (seasonIds.length || this.existingProduct.tags.seasons.length === 0) {
      patch['season_ids'] = seasonIds;
    }

    if (this.existingProduct.purchase_location) {
      const purchaseLocationId = this.purchaseLocations.find(
        location => location.name.toLowerCase() === this.existingProduct!.purchase_location!.toLowerCase(),
      )?.id;
      if (purchaseLocationId) {
        patch['purchase_location_id'] = purchaseLocationId;
      }
    }

    if (Object.keys(patch).length > 0) {
      this.form.patchValue(patch, { emitEvent: false });
    }
  }

  onCollabChange(): void {
    this.cdr.markForCheck();
  }

  openBrandModal(): void {
    this.brandModalVisible.set(true);
  }

  onBrandModalVisibleChange(next: boolean): void {
    this.brandModalVisible.set(next);
  }

  onBrandCreated(brand: Brand): void {
    this.brands = [...this.brands, brand].sort((a, b) => a.name.localeCompare(b.name));
    this.form.get('brand_id')?.setValue(brand.id);
    this.brandModalVisible.set(false);
    this.cdr.markForCheck();
  }

  async onSubmit(): Promise<void> {
    this.form.markAllAsTouched();
    if (this.form.invalid) return;

    this.saving.set(true);
    this.staleVersionError.set(false);

    try {
      const fv = this.form.getRawValue();
      const typeDetailsInput = this.buildTypeDetailsInput(fv);
      const sizeInput = {
        size_value: fv.size.size_value || undefined,
        size_value2: fv.size.size_value2 || undefined,
        size_system: fv.size.size_system || undefined,
        measurement_cm: fv.size.measurement_cm || undefined,
      };

      if (this.productId()) {
        // Edit mode
        const updateReq: UpdateProductRequest = {
          name: fv.name,
          brand_id: fv.brand_id,
          category_id: fv.category_id,
          status: fv.status || undefined,
          purchase_price: fv.purchase_price || undefined,
          purchase_location_id: fv.purchase_location_id || undefined,
          currency: fv.currency || undefined,
          ai_notes: fv.ai_notes || undefined,
          details: {
            condition: fv.details.condition,
            material: fv.details.material || undefined,
            color: fv.details.color || undefined,
            year_of_release: fv.details.year_of_release || undefined,
            is_vintage: fv.details.is_vintage,
            is_collab: fv.details.is_collab,
            collab_name: fv.details.collab_name || undefined,
            is_limited_edition: fv.details.is_limited_edition,
            special_notes: fv.details.special_notes || undefined,
            size: sizeInput,
            type_details: typeDetailsInput,
          },
          style_tag_ids: fv.style_tag_ids ?? [],
          vibe_tag_ids: fv.vibe_tag_ids ?? [],
          season_ids: fv.season_ids ?? [],
          expected_version: this.existingProduct?.version ?? 1,
        };

        const res = await firstValueFrom(this.productApi.updateProduct(this.productId()!, updateReq));

        if (!res.success) {
          if (res.error?.toLowerCase().includes('version') || res.error?.toLowerCase().includes('conflict')) {
            this.staleVersionError.set(true);
          } else {
            this.messageService.add({
              severity: 'error',
              summary: this.t('admin.products.form.messages.updateFailedSummary', this.t('common.error')),
              detail: res.error ?? this.t('admin.products.form.messages.updateFailedDetail', 'Failed to update product'),
            });
          }
          return;
        }

        this.saved.emit();
      } else {
        // Create mode
        const createReq: CreateProductRequest = {
          name: fv.name,
          brand_id: fv.brand_id,
          category_id: fv.category_id,
          status: fv.status || undefined,
          purchase_price: fv.purchase_price || undefined,
          purchase_location_id: fv.purchase_location_id || undefined,
          currency: fv.currency || undefined,
          ai_notes: fv.ai_notes || undefined,
          details: {
            condition: fv.details.condition,
            material: fv.details.material || undefined,
            color: fv.details.color || undefined,
            year_of_release: fv.details.year_of_release || undefined,
            is_vintage: fv.details.is_vintage,
            is_collab: fv.details.is_collab,
            collab_name: fv.details.collab_name || undefined,
            is_limited_edition: fv.details.is_limited_edition,
            special_notes: fv.details.special_notes || undefined,
            size: sizeInput,
            type_details: typeDetailsInput,
          },
          style_tag_ids: fv.style_tag_ids ?? [],
          vibe_tag_ids: fv.vibe_tag_ids ?? [],
          season_ids: fv.season_ids ?? [],
        };

        const res = await firstValueFrom(this.productApi.createProduct(createReq));

        if (!res.success) {
          this.messageService.add({
            severity: 'error',
            summary: this.t('admin.products.form.messages.createFailedSummary', this.t('common.error')),
            detail: res.error ?? this.t('admin.products.form.messages.createFailedDetail', 'Failed to create product'),
          });
          return;
        }

        this.saved.emit();
      }
    } catch (err: any) {
      const errMsg = err.error?.error ?? err.message ?? this.t('admin.products.form.messages.genericError', 'An error occurred');
      if (err.status === 409) {
        this.staleVersionError.set(true);
      } else {
        this.messageService.add({
          severity: 'error',
          summary: this.t('admin.products.form.messages.saveFailedSummary', this.t('common.error')),
          detail: errMsg,
        });
      }
    } finally {
      this.saving.set(false);
    }
  }

  private buildTypeDetailsInput(fv: any): any {
    const pt = this.selectedProductType();
    if (!pt) return { product_type: 'accessories' };
    const td = fv.type_details;

    switch (pt) {
      case 'clothing':
        return { product_type: 'clothing', fit: td.fit || undefined };
      case 'footwear':
        return { product_type: 'footwear', shoe_width: td.shoe_width || undefined, insole_length_cm: td.insole_length_cm || undefined };
      case 'bags':
        return { product_type: 'bags', width_cm: td.width_cm || undefined, height_cm: td.height_cm || undefined, depth_cm: td.depth_cm || undefined, handle_type: td.handle_type || undefined, bag_size_label: td.bag_size_label || undefined };
      case 'jewelry':
        return { product_type: 'jewelry', metal: td.metal || undefined, stone: td.stone || undefined, clasp_type: td.clasp_type || undefined };
      default:
        return { product_type: 'accessories' };
    }
  }

  onCancel(): void {
    this.visibleChange.emit(false);
  }

  onManageImages(): void {
    const id = this.productId();
    if (!id) return;
    this.router.navigate(['/admin/products', id, 'images']);
  }

  getTypeLabel(type: string): string {
    return this.t(`productTypes.${type}`, type);
  }

  private t(key: string, fallback?: string, params?: Record<string, string | number>): string {
    const translated = this.translate.instant(key, params);
    return translated === key ? (fallback ?? key) : translated;
  }
}
