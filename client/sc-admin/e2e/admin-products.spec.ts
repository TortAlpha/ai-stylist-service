import { expect, test } from '@playwright/test';
import {
  mockAdminIdentityApis,
  mockAdminProductsApis,
  seedAdminSession,
  seededProduct,
} from './fixtures/admin-mocks';

test.describe('Admin products', () => {
  test('shows list data and navigates to product detail', async ({ page }) => {
    await seedAdminSession(page);
    await mockAdminIdentityApis(page);
    await mockAdminProductsApis(page);

    await page.goto('/admin/products?view=table');
    await expect(page).toHaveURL(/\/admin\/products/);
    await expect(page.locator('body')).toContainText(seededProduct.name);
    await expect(page.locator('body')).toContainText(seededProduct.sku);

    await page.locator('button.name-link', { hasText: seededProduct.name }).click();

    await page.waitForURL(`**/admin/products/${seededProduct.id}`);
    await expect(page.getByRole('heading', { name: seededProduct.name })).toBeVisible();
    await expect(page.locator('body')).toContainText(seededProduct.sku);
  });

  test('syncs search filter to URL and API request', async ({ page }) => {
    await seedAdminSession(page);
    await mockAdminIdentityApis(page);
    const { searchTerms } = await mockAdminProductsApis(page);

    await page.goto('/admin/products');
    await page.getByPlaceholder('Search by name or SKU').fill('jacket');

    await expect.poll(() => searchTerms.at(-1)).toBe('jacket');
    await expect(page).toHaveURL(/search=jacket/);

    await page.getByRole('button', { name: 'Clear Filters' }).click();
    await expect(page).not.toHaveURL(/search=/);
  });

  test('toggles sort order with button and syncs URL/API', async ({ page }) => {
    await seedAdminSession(page);
    await mockAdminIdentityApis(page);
    const { sortOrders } = await mockAdminProductsApis(page);

    await page.goto('/admin/products');

    const orderButton = page.locator('.sort-order-field button').first();
    await expect(orderButton).toBeVisible();
    await orderButton.click();

    await expect(page).toHaveURL(/sort_order=asc/);
    await expect.poll(() => sortOrders.at(-1)).toBe('asc');
  });
});
