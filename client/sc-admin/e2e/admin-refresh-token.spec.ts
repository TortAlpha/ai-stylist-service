import { expect, test } from '@playwright/test';
import { createAccessToken, mockAdminIdentityApis, seedAdminSession } from './fixtures/admin-mocks';

const NOW = '2026-01-01T00:00:00.000Z';

test.describe('Auth refresh flow', () => {
  test('401 on protected call triggers refresh and retries transparently', async ({ page }) => {
    await seedAdminSession(page);
    await mockAdminIdentityApis(page);

    const newToken = createAccessToken('admin-1').replace('.', '-new.');
    let refreshCount = 0;
    let productsCalls = 0;

    await page.route('**/api/auth/refresh', route => {
      refreshCount += 1;
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ access_token: newToken, refresh_token: 'refresh-token-new' }),
      });
    });

    await page.route('**/api/brands', route => route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ success: true, data: [], error: null }),
    }));

    await page.route('**/api/admin/products*', route => {
      productsCalls += 1;
      const auth = route.request().headers()['authorization'] ?? '';
      if (productsCalls === 1) {
        return route.fulfill({ status: 401, contentType: 'application/json', body: '{"error":"unauth"}' });
      }
      // On retry, verify Authorization carries the refreshed token
      expect(auth).toContain('-new.');
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          success: true,
          data: { items: [], total: 0, page: 1, per_page: 20, total_pages: 0 },
          error: null,
        }),
      });
    });

    await page.goto('/admin/products');

    await expect.poll(() => refreshCount).toBeGreaterThanOrEqual(1);
    await expect.poll(() => productsCalls).toBeGreaterThanOrEqual(2);

    // Page should render products page (not bounce to login)
    await expect(page).toHaveURL(/\/admin\/products/);
  });

  test('failed refresh logs out user and routes to login', async ({ page }) => {
    await seedAdminSession(page);
    await mockAdminIdentityApis(page);

    await page.route('**/api/brands', route => route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ success: true, data: [], error: null }),
    }));

    await page.route('**/api/auth/refresh', route => route.fulfill({
      status: 401,
      contentType: 'application/json',
      body: '{"error":"refresh failed"}',
    }));

    await page.route('**/api/admin/products*', route => route.fulfill({
      status: 401,
      contentType: 'application/json',
      body: '{"error":"unauth"}',
    }));

    await page.goto('/admin/products');

    // After failed refresh the app should eventually route to login
    await page.waitForURL('**/login', { timeout: 10_000 });
  });
});
