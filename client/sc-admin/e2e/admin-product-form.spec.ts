import { expect, test } from '@playwright/test';
import {
  mockAdminBrandsApis,
  mockAdminIdentityApis,
  mockAdminProductsApis,
  mockEmptyListingsApis,
  mockProductFormSupportApis,
  seedAdminSession,
} from './fixtures/admin-mocks';

async function setupProductFormPage(page: import('@playwright/test').Page) {
  await seedAdminSession(page);
  await mockAdminIdentityApis(page);
  const brandState = await mockAdminBrandsApis(page);
  await mockAdminProductsApis(page);
  await mockProductFormSupportApis(page);
  await mockEmptyListingsApis(page);
  return { brandState };
}

test.describe('Product form', () => {
  test('blocks submit until required fields are filled', async ({ page }) => {
    await setupProductFormPage(page);

    await page.goto('/admin/products/new');
    const submitBtn = page.locator('button:has-text("Create Product"), button:has-text("Создать товар")').first();
    await expect(submitBtn).toBeDisabled();
  });

  test('quick-adds a brand from inside the product form', async ({ page }) => {
    const { brandState } = await setupProductFormPage(page);

    await page.goto('/admin/products/new');

    // Click the + button next to the brand select.
    const addBrandBtn = page.locator('app-product-form-modal .brand-select-row button');
    await addBrandBtn.first().click();

    // Brand modal should appear
    await expect(page.locator('app-brand-form-modal .p-dialog')).toBeVisible();
    await page.locator('app-brand-form-modal #name').fill('QuickAdd Brand');
    await page.locator('app-brand-form-modal #code').fill('QA');

    await page.locator('app-brand-form-modal button:has-text("Create Brand"), app-brand-form-modal button:has-text("Создать бренд")').first().click();

    // Verify API was called
    await expect.poll(() => brandState.creates.length).toBeGreaterThan(0);
    expect(brandState.creates[0].name).toBe('QuickAdd Brand');

    // Modal should close
    await expect(page.locator('app-brand-form-modal .p-dialog')).toBeHidden();
  });
});
