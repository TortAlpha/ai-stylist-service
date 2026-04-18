import type { Page } from '@playwright/test';

const ADMIN_USER_ID = 'admin-1';
const NOW = '2026-01-01T00:00:00.000Z';

export const seededProduct = {
  id: 'prod-1',
  sku: 'SKU-0001',
  name: 'Test Jacket',
  purchase_price: '120.00',
  currency: 'EUR',
  preview_url: null,
  brand_name: 'Acme',
  product_type: 'clothing',
  category: 'Outerwear',
  status: 'ready',
  condition: 'excellent',
  color: 'Black',
  size: {
    size_group: 'clothing',
    size_value: 'M',
  },
};

function createProductDetail(productId: string) {
  return {
    id: productId,
    sku: seededProduct.sku,
    name: seededProduct.name,
    purchase_price: seededProduct.purchase_price,
    purchase_location: 'Belgrade Warehouse',
    currency: seededProduct.currency,
    ai_notes: null,
    preview_url: seededProduct.preview_url,
    image_urls: [],
    brand: { id: 1, name: seededProduct.brand_name, tier: 'premium' },
    brand_id: 1,
    product_type: seededProduct.product_type,
    category: {
      name: seededProduct.category,
      parent_category: 'Jackets',
      gender: 'unisex',
      size_group: 'clothing',
    },
    category_id: 10,
    status: seededProduct.status,
    version: 3,
    details: {
      material: 'Wool',
      condition: seededProduct.condition,
      color: seededProduct.color,
      year_of_release: 2021,
      is_vintage: false,
      is_collab: false,
      collab_name: null,
      is_limited_edition: false,
      special_notes: null,
    },
    size: {
      size_group: 'clothing',
      size_value: 'M',
      size_label: 'M',
    },
    type_details: {
      type: 'clothing',
      fit: 'regular',
    },
    tags: {
      styles: ['minimal'],
      vibes: ['casual'],
      seasons: ['winter'],
    },
    created_at: NOW,
    updated_at: NOW,
  };
}

export function createAccessToken(userId = ADMIN_USER_ID): string {
  const payload = Buffer.from(JSON.stringify({ sub: userId }), 'utf8').toString('base64');
  return `header.${payload}.signature`;
}

export async function seedAdminSession(page: Page, accessToken = createAccessToken()): Promise<void> {
  await page.addInitScript(({ token }) => {
    window.localStorage.setItem('access_token', token);
    window.localStorage.setItem('refresh_token', 'refresh-token');
  }, { token: accessToken });
}

export async function mockAdminIdentityApis(page: Page, accessToken = createAccessToken()): Promise<void> {
  await page.route('**/api/auth/login', route => route.fulfill({
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify({
      access_token: accessToken,
      refresh_token: 'refresh-token',
    }),
  }));

  await page.route('**/api/auth/refresh', route => route.fulfill({
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify({
      access_token: accessToken,
      refresh_token: 'refresh-token',
    }),
  }));

  await page.route(`**/api/users/${ADMIN_USER_ID}`, route => route.fulfill({
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify({
      id: ADMIN_USER_ID,
      name: 'Admin',
      surname: 'User',
      email: 'admin@example.com',
      phone_number: null,
      is_active: true,
      role: 'admin',
      created_at: NOW,
      updated_at: NOW,
    }),
  }));
}

export interface BrandRecord {
  id: number;
  name: string;
  code: string;
  tier: string;
  country: string | null;
  created_at: string;
}

export interface BrandMocks {
  brands: BrandRecord[];
  searchTerms: string[];
  creates: Array<{ name: string; code: string; tier: string }>;
  updates: Array<{ id: number; name?: string }>;
  deletes: number[];
}

export async function mockAdminBrandsApis(page: Page, seed?: BrandRecord[]): Promise<BrandMocks> {
  const state: BrandMocks = {
    brands: seed ?? [
      { id: 1, name: 'Acme', code: 'ACME', tier: 'premium', country: 'RS', created_at: NOW },
      { id: 2, name: 'Zenith', code: 'ZNTH', tier: 'luxury', country: 'IT', created_at: NOW },
    ],
    searchTerms: [],
    creates: [],
    updates: [],
    deletes: [],
  };

  await page.route('**/api/brands', route => route.fulfill({
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify({ success: true, data: state.brands, error: null }),
  }));

  await page.route('**/api/admin/brands/*', async route => {
    const url = new URL(route.request().url());
    const match = url.pathname.match(/\/api\/admin\/brands\/(\d+)$/);
    const id = match ? Number(match[1]) : null;
    if (!id) return route.fulfill({ status: 404, body: 'Not found' });

    const method = route.request().method();
    if (method === 'GET') {
      const found = state.brands.find(b => b.id === id);
      if (!found) return route.fulfill({ status: 404, body: 'Not found' });
      return route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ success: true, data: found, error: null }) });
    }
    if (method === 'PUT') {
      const body = JSON.parse(route.request().postData() ?? '{}');
      state.updates.push({ id, ...body });
      const existing = state.brands.find(b => b.id === id);
      if (existing) Object.assign(existing, body);
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ success: true, data: existing ?? { id, ...body, created_at: NOW }, error: null }),
      });
    }
    if (method === 'DELETE') {
      state.deletes.push(id);
      state.brands = state.brands.filter(b => b.id !== id);
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ success: true, data: null, error: null }),
      });
    }
    return route.fulfill({ status: 405, body: 'method not allowed' });
  });

  await page.route('**/api/admin/brands*', async route => {
    const url = new URL(route.request().url());
    if (url.pathname !== '/api/admin/brands') return route.fulfill({ status: 404, body: 'Not found' });

    const method = route.request().method();
    if (method === 'GET') {
      const search = url.searchParams.get('search')?.trim().toLowerCase() ?? '';
      if (search) state.searchTerms.push(search);
      const items = state.brands.filter(b =>
        !search || b.name.toLowerCase().includes(search) || b.code.toLowerCase().includes(search),
      );
      const pageN = Number(url.searchParams.get('page') ?? '1');
      const perPage = Number(url.searchParams.get('per_page') ?? '20');
      return route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          success: true,
          data: {
            items,
            total: items.length,
            page: Number.isFinite(pageN) && pageN > 0 ? pageN : 1,
            per_page: Number.isFinite(perPage) && perPage > 0 ? perPage : 20,
            total_pages: items.length > 0 ? 1 : 0,
          },
          error: null,
        }),
      });
    }
    if (method === 'POST') {
      const body = JSON.parse(route.request().postData() ?? '{}');
      state.creates.push(body);
      const created: BrandRecord = {
        id: Math.max(0, ...state.brands.map(b => b.id)) + 1,
        name: body.name,
        code: body.code,
        tier: body.tier,
        country: body.country ?? null,
        created_at: NOW,
      };
      state.brands.push(created);
      return route.fulfill({
        status: 201,
        contentType: 'application/json',
        body: JSON.stringify({ success: true, data: created, error: null }),
      });
    }
    return route.fulfill({ status: 405, body: 'method not allowed' });
  });

  return state;
}

export async function mockProductFormSupportApis(page: Page): Promise<void> {
  await page.route('**/api/categories*', route => route.fulfill({
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify({
      success: true,
      data: [
        {
          id: 10,
          name: 'Jackets',
          parent_id: null,
          gender: 'unisex',
          product_type: 'clothing',
          size_group: 'letter',
        },
      ],
      error: null,
    }),
  }));

  const tagResponses: Record<string, unknown[]> = {
    styles: [{ id: 1, name: 'minimal' }],
    vibes: [{ id: 2, name: 'casual' }],
    seasons: [{ id: 3, name: 'winter' }],
  };
  for (const type of Object.keys(tagResponses)) {
    await page.route(`**/api/tags/${type}`, route => route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ success: true, data: tagResponses[type], error: null }),
    }));
  }

  await page.route('**/api/admin/purchase-locations', route => route.fulfill({
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify({
      success: true,
      data: [{ id: 1, name: 'Belgrade Warehouse', created_at: NOW }],
      error: null,
    }),
  }));
}

export async function mockEmptyListingsApis(page: Page): Promise<void> {
  await page.route('**/api/listings/marketplaces', route => route.fulfill({
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify({ success: true, data: [], error: null }),
  }));

  await page.route('**/api/listings*', route => {
    const url = new URL(route.request().url());
    if (!url.pathname.startsWith('/api/listings')) {
      return route.fulfill({ status: 404, body: 'Not found' });
    }
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
}

export async function mockAdminProductsApis(page: Page): Promise<{ searchTerms: string[]; sortOrders: string[] }> {
  const searchTerms: string[] = [];
  const sortOrders: string[] = [];

  await page.route('**/api/brands', route => route.fulfill({
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify({
      success: true,
      data: [
        { id: 1, name: 'Acme', code: 'ACME', tier: 'premium', country: 'RS', created_at: NOW },
      ],
      error: null,
    }),
  }));

  await page.route('**/api/admin/products/*', route => {
    const requestUrl = new URL(route.request().url());
    const match = requestUrl.pathname.match(/\/api\/admin\/products\/([^/]+)$/);
    const productId = match?.[1];
    if (!productId) {
      return route.fulfill({ status: 404, body: 'Not found' });
    }

    return route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        success: true,
        data: createProductDetail(productId),
        error: null,
      }),
    });
  });

  await page.route('**/api/admin/products*', route => {
    const requestUrl = new URL(route.request().url());
    if (requestUrl.pathname !== '/api/admin/products') {
      return route.fulfill({ status: 404, body: 'Not found' });
    }

    const rawSearch = requestUrl.searchParams.get('search') ?? '';
    const search = rawSearch.trim().toLowerCase();
    if (search) {
      searchTerms.push(search);
    }

    const sortOrder = requestUrl.searchParams.get('sort_order') ?? 'desc';
    sortOrders.push(sortOrder);

    const items = [seededProduct].filter(product => {
      if (!search) return true;
      return (
        product.name.toLowerCase().includes(search) ||
        product.sku.toLowerCase().includes(search)
      );
    });

    const pageParam = Number(requestUrl.searchParams.get('page') ?? '1');
    const perPageParam = Number(requestUrl.searchParams.get('per_page') ?? '20');

    return route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        success: true,
        data: {
          items,
          total: items.length,
          page: Number.isFinite(pageParam) && pageParam > 0 ? pageParam : 1,
          per_page: Number.isFinite(perPageParam) && perPageParam > 0 ? perPageParam : 20,
          total_pages: items.length > 0 ? 1 : 0,
        },
        error: null,
      }),
    });
  });

  return { searchTerms, sortOrders };
}
