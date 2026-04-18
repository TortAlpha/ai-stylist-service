import { expect, test } from '@playwright/test';
import {
  mockAdminIdentityApis,
  mockAdminProductsApis,
  mockEmptyListingsApis,
  seedAdminSession,
} from './fixtures/admin-mocks';

test.describe('Admin listings', () => {
  test('shows listings page (empty state)', async ({ page }) => {
    await seedAdminSession(page);
    await mockAdminIdentityApis(page);
    await mockAdminProductsApis(page);
    await mockEmptyListingsApis(page);

    await page.goto('/admin/listings');
    await expect(page).toHaveURL(/\/admin\/listings/);
  });
});

test.describe('Admin profile', () => {
  test('shows profile page with user data', async ({ page }) => {
    await seedAdminSession(page);
    await mockAdminIdentityApis(page);
    await mockAdminProductsApis(page);

    // Addresses endpoint for profile
    await page.route('**/api/users/*/addresses', route => route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify([]),
    }));

    await page.goto('/admin/profile');
    await expect(page).toHaveURL(/\/admin\/profile/);
    await expect(page.locator('#email')).toHaveValue('admin@example.com');
  });
});
