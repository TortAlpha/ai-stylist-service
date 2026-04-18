import { expect, test } from '@playwright/test';
import {
  mockAdminBrandsApis,
  mockAdminIdentityApis,
  seedAdminSession,
} from './fixtures/admin-mocks';

test.describe('Admin brands', () => {
  test('shows list of seeded brands', async ({ page }) => {
    await seedAdminSession(page);
    await mockAdminIdentityApis(page);
    await mockAdminBrandsApis(page);

    await page.goto('/admin/brands');
    await expect(page).toHaveURL(/\/admin\/brands/);
    await expect(page.locator('body')).toContainText('Acme');
    await expect(page.locator('body')).toContainText('Zenith');
  });

  test('creates a new brand', async ({ page }) => {
    await seedAdminSession(page);
    await mockAdminIdentityApis(page);
    const state = await mockAdminBrandsApis(page);

    await page.goto('/admin/brands/new');
    await page.locator('#name').fill('Prada');
    await page.locator('#code').fill('PRADA');

    await page.locator('button:has-text("Create Brand"), button:has-text("Создать бренд")').first().click();

    await page.waitForURL('**/admin/brands');
    expect(state.creates.length).toBe(1);
    expect(state.creates[0].name).toBe('Prada');
    expect(state.creates[0].code).toBe('PRADA');
  });

  test('edits an existing brand', async ({ page }) => {
    await seedAdminSession(page);
    await mockAdminIdentityApis(page);
    const state = await mockAdminBrandsApis(page);

    await page.goto('/admin/brands/1/edit');
    await page.locator('#name').fill('Acme Renamed');

    await page.locator('button:has-text("Save Changes"), button:has-text("Сохранить изменения")').first().click();

    await page.waitForURL('**/admin/brands');
    expect(state.updates.length).toBe(1);
    expect(state.updates[0].id).toBe(1);
    expect(state.updates[0].name).toBe('Acme Renamed');
  });

  test('filters brands via search and syncs URL', async ({ page }) => {
    await seedAdminSession(page);
    await mockAdminIdentityApis(page);
    const state = await mockAdminBrandsApis(page);

    await page.goto('/admin/brands');
    await page.getByPlaceholder(/search|поиск/i).first().fill('Zen');

    await expect.poll(() => state.searchTerms.at(-1)).toBe('zen');
    await expect(page).toHaveURL(/search=Zen/);
  });
});
