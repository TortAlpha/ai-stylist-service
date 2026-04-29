import { ComponentRef } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { provideNoopAnimations } from '@angular/platform-browser/animations';
import { provideRouter } from '@angular/router';
import { TranslateModule } from '@ngx-translate/core';
import { TreeNode } from 'primeng/api';
import { of } from 'rxjs';
import { describe, expect, it, beforeEach, vi } from 'vitest';
import { ProductFormModalComponent } from './product-form-modal.component';
import { ProductApiService } from '../../../core/services/product-api.service';
import { BrandApiService } from '../../../core/services/brand-api.service';
import { CategoryApiService } from '../../../core/services/category-api.service';
import { TagApiService } from '../../../core/services/tag-api.service';
import { PurchaseLocationApiService } from '../../../core/services/purchase-location-api.service';
import { Brand } from '../../../core/models/brand.model';

const NOW = '2026-01-01T00:00:00.000Z';

const clothingCategory = {
  id: 10,
  name: 'Jackets',
  parent_id: null,
  gender: 'unisex',
  product_type: 'clothing',
  size_group: 'letter',
};

const bagsCategory = {
  id: 20,
  name: 'Tote',
  parent_id: null,
  gender: 'women',
  product_type: 'bags',
  size_group: 'dimensions',
};

const jewelryCategory = {
  id: 30,
  name: 'Rings',
  parent_id: null,
  gender: 'unisex',
  product_type: 'jewelry',
  size_group: 'ring',
};

const brandAcme: Brand = { id: 1, name: 'Acme', code: 'ACME', tier: 'premium', country: null, created_at: NOW };

function arrayRes<T>(data: T[]) {
  return of({ success: true, data, error: null });
}

function collectSelectableCategoryIds(nodes: TreeNode[]): number[] {
  return nodes.flatMap(node => [
    ...((node.data as { id?: number } | undefined)?.id ? [(node.data as { id: number }).id] : []),
    ...collectSelectableCategoryIds(node.children ?? []),
  ]);
}

function findCategoryNode(nodes: TreeNode[], id: number): TreeNode | null {
  for (const node of nodes) {
    if ((node.data as { id?: number } | undefined)?.id === id) {
      return node;
    }
    const child = findCategoryNode(node.children ?? [], id);
    if (child) {
      return child;
    }
  }
  return null;
}

describe('ProductFormModalComponent', () => {
  let fixture: ComponentFixture<ProductFormModalComponent>;
  let ref: ComponentRef<ProductFormModalComponent>;
  let component: ProductFormModalComponent;
  let productApi: {
    createProduct: ReturnType<typeof vi.fn>;
    updateProduct: ReturnType<typeof vi.fn>;
    getProduct: ReturnType<typeof vi.fn>;
  };

  beforeEach(() => {
    productApi = {
      createProduct: vi.fn().mockReturnValue(of({ success: true, data: {}, error: null })),
      updateProduct: vi.fn().mockReturnValue(of({ success: true, data: {}, error: null })),
      getProduct: vi.fn().mockReturnValue(of({ success: true, data: {}, error: null })),
    };

    TestBed.configureTestingModule({
      imports: [ProductFormModalComponent, TranslateModule.forRoot()],
      providers: [
        provideNoopAnimations(),
        provideRouter([]),
        { provide: ProductApiService, useValue: productApi },
        { provide: BrandApiService, useValue: { getAll: () => arrayRes([brandAcme]) } },
        { provide: CategoryApiService, useValue: { getAll: () => arrayRes([clothingCategory, bagsCategory]) } },
        {
          provide: TagApiService,
          useValue: {
            getStyles: () => arrayRes([{ id: 11, name: 'minimal' }]),
            getVibes: () => arrayRes([{ id: 21, name: 'casual' }]),
            getSeasons: () => arrayRes([{ id: 31, name: 'winter' }]),
          },
        },
        {
          provide: PurchaseLocationApiService,
          useValue: { getAll: () => arrayRes([{ id: 1, name: 'Belgrade Warehouse', created_at: NOW }]) },
        },
      ],
    });

    fixture = TestBed.createComponent(ProductFormModalComponent);
    ref = fixture.componentRef;
    component = fixture.componentInstance;
    ref.setInput('visible', false);
  });

  it('required-field validation: name, brand, category, condition', () => {
    expect(component.form.valid).toBe(false);

    component.form.patchValue({ name: 'Jacket', brand_id: 1, category_id: 10 });
    component.form.get('details')?.patchValue({ condition: 'excellent' });

    expect(component.form.valid).toBe(true);
  });

  it('category change sets product_type and size_group signals', () => {
    component.categories = [clothingCategory, bagsCategory] as any;
    component.form.get('category_id')?.setValue(20);
    component.onCategoryChange();
    expect(component['selectedProductType']()).toBe('bags');
    expect(component['selectedSizeGroup']()).toBe('dimensions');
  });

  it('does not render a separate type details section for jewelry', () => {
    component['selectedProductType'].set('jewelry');
    expect(component['showSeparateTypeDetailsSection']()).toBe(false);

    component['selectedProductType'].set('bags');
    expect(component['showSeparateTypeDetailsSection']()).toBe(true);
  });

  it('categoryTreeNodes allows leaf categories only', () => {
    component.categories = [
      { ...clothingCategory, id: 1, parent_id: null, name: 'Root' },
      { ...clothingCategory, id: 2, parent_id: 1, name: 'Leaf' },
    ] as any;
    component.form.get('category_id')?.setValue(null);

    expect(collectSelectableCategoryIds(component.categoryTreeNodes)).toEqual([2]);
  });

  it('category tree selection writes category_id and derived signals', () => {
    component.categories = [clothingCategory, bagsCategory] as any;

    const node = findCategoryNode(component.categoryTreeNodes, 20);
    component.onCategoryNodeChange(node);

    expect(component.form.get('category_id')?.value).toBe(20);
    expect(component['selectedProductType']()).toBe('bags');
    expect(component['selectedSizeGroup']()).toBe('dimensions');
  });

  it('category tree hides root category when it duplicates product type', () => {
    component.categories = [
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
    ] as any;

    const typeNode = findCategoryNode(component.categoryTreeNodes, 2);
    expect(typeNode?.label).toBe('Crossbody Bags');
    expect(component.categoryTreeNodes[0].children?.[0].children?.map(node => node.label)).toEqual([
      'Crossbody Bags',
    ]);

    component.form.get('category_id')?.setValue(2);
    expect(component.selectedCategoryPath).toBe('Female / Bags / Crossbody Bags');
  });

  it('onBrandCreated appends to brands, sorts, and auto-selects', () => {
    component.brands = [brandAcme];
    const newBrand: Brand = { id: 2, name: 'Aardvark', code: 'AA', tier: 'mass', country: null, created_at: NOW };

    component.onBrandCreated(newBrand);

    expect(component.brands.map(b => b.name)).toEqual(['Aardvark', 'Acme']);
    expect(component.form.get('brand_id')?.value).toBe(2);
    expect(component['brandModalVisible']()).toBe(false);
  });

  it('openBrandModal sets signal true', () => {
    component.openBrandModal();
    expect(component['brandModalVisible']()).toBe(true);
  });

  it('submit (create) sends CreateProductRequest with clothing type_details', async () => {
    component.brands = [brandAcme];
    component.categories = [clothingCategory] as any;
    component.form.patchValue({
      name: 'Jacket',
      brand_id: 1,
      category_id: 10,
      status: 'ready',
      purchase_price: '100',
    });
    component.form.get('details')?.patchValue({ condition: 'excellent' });
    component.form.get('size')?.patchValue({ size_value: 'M' });
    component['selectedProductType'].set('clothing');
    component.form.get('type_details')?.patchValue({ fit: 'regular' });

    await component.onSubmit();

    expect(productApi.createProduct).toHaveBeenCalledTimes(1);
    const body = productApi.createProduct.mock.calls[0][0];
    expect(body.name).toBe('Jacket');
    expect(body.brand_id).toBe(1);
    expect(body.category_id).toBe(10);
    expect(body.details.condition).toBe('excellent');
    expect(body.details.size.size_value).toBe('M');
    expect(body.details.type_details).toEqual({ product_type: 'clothing', fit: 'regular' });
  });

  it('submit (create) still sends jewelry type_details from inline details fields', async () => {
    component.brands = [brandAcme];
    component.categories = [jewelryCategory] as any;
    component.form.patchValue({
      name: 'Ring',
      brand_id: 1,
      category_id: 30,
    });
    component.form.get('details')?.patchValue({ condition: 'excellent' });
    component['selectedProductType'].set('jewelry');
    component.form.get('type_details')?.patchValue({
      metal: 'gold',
      stone: 'diamond',
      clasp_type: 'lobster',
    });

    await component.onSubmit();

    const body = productApi.createProduct.mock.calls[0][0];
    expect(body.details.type_details).toEqual({
      product_type: 'jewelry',
      metal: 'gold',
      stone: 'diamond',
      clasp_type: 'lobster',
    });
  });

  it('submit (create) falls back to accessories type_details when no product type', async () => {
    component.brands = [brandAcme];
    component.categories = [clothingCategory] as any;
    component.form.patchValue({ name: 'X', brand_id: 1, category_id: 10 });
    component.form.get('details')?.patchValue({ condition: 'good' });
    component['selectedProductType'].set(null);

    await component.onSubmit();

    const body = productApi.createProduct.mock.calls[0][0];
    expect(body.details.type_details).toEqual({ product_type: 'accessories' });
  });

  it('submit (update) sends UpdateProductRequest with expected_version', async () => {
    ref.setInput('productId', 'p-1');
    component.brands = [brandAcme];
    component.categories = [clothingCategory] as any;
    component.existingProduct = { version: 7 } as any;
    component.form.patchValue({ name: 'X', brand_id: 1, category_id: 10 });
    component.form.get('details')?.patchValue({ condition: 'good' });
    component['selectedProductType'].set('clothing');

    await component.onSubmit();

    expect(productApi.updateProduct).toHaveBeenCalledTimes(1);
    const [id, body] = productApi.updateProduct.mock.calls[0];
    expect(id).toBe('p-1');
    expect(body.expected_version).toBe(7);
  });

  it('does not submit when form is invalid', async () => {
    await component.onSubmit();
    expect(productApi.createProduct).not.toHaveBeenCalled();
  });

  it('onCancel emits visibleChange false', () => {
    const spy = vi.fn();
    component.visibleChange.subscribe(spy);
    component.onCancel();
    expect(spy).toHaveBeenCalledWith(false);
  });
});
