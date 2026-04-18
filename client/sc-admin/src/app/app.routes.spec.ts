import { routes } from './app.routes';
import { adminAuthGuard } from './core/guards/admin-auth.guard';

describe('App routes', () => {
  it('has root and wildcard redirects to admin products', () => {
    const root = routes.find(route => route.path === '');
    const wildcard = routes.find(route => route.path === '**');

    expect(root?.redirectTo).toBe('admin/products');
    expect(root?.pathMatch).toBe('full');
    expect(wildcard?.redirectTo).toBe('admin/products');
  });

  it('contains login and admin routes', () => {
    const login = routes.find(route => route.path === 'login');
    const admin = routes.find(route => route.path === 'admin');

    expect(login?.loadChildren).toBeTypeOf('function');
    expect(admin?.loadComponent).toBeTypeOf('function');
    expect(admin?.loadChildren).toBeTypeOf('function');
    expect(admin?.canActivate).toContain(adminAuthGuard);
  });
});
