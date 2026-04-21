import { expect, test } from '@playwright/test';
import { mockAdminIdentityApis, mockAdminProductsApis } from './fixtures/admin-mocks';

test.describe('Admin auth', () => {
  test('redirects unauthenticated user to login', async ({ page }) => {
    await page.goto('/admin/products');

    await page.waitForURL('**/login');
    await expect(page.locator('form')).toBeVisible();
    await expect(page.locator('button[type="submit"]')).toBeVisible();
  });

  test('logs in admin user and opens products page', async ({ page }) => {
    await mockAdminIdentityApis(page);
    await mockAdminProductsApis(page);

    await page.goto('/login');
    await page.locator('#email').fill('admin@example.com');
    await page.locator('#password').fill('secret');
    await page.locator('button[type="submit"]').click();

    await page.waitForURL('**/admin/products');
    await expect(page.getByRole('heading', { name: /Admin Panel/i })).toBeVisible();
  });
});
