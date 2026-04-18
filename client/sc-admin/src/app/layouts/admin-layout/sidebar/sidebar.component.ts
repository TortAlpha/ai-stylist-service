import { Component, ChangeDetectionStrategy, OnDestroy, OnInit, inject } from '@angular/core';
import { TranslateService } from '@ngx-translate/core';
import { MenubarModule } from 'primeng/menubar';
import { MenuItem } from 'primeng/api';
import { Subscription } from 'rxjs';


@Component({
  selector: 'app-sidebar',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [MenubarModule],
  templateUrl: './sidebar.component.html',
  styleUrl: './sidebar.component.scss',
})
export class SidebarComponent implements OnInit, OnDestroy {
  private translate = inject(TranslateService);
  private langChangeSub: Subscription | null = null;

  menuItems: MenuItem[] = [];

  ngOnInit(): void {
    this.buildMenuItems();
    this.langChangeSub = this.translate.onLangChange.subscribe(() => {
      this.buildMenuItems();
    });
  }

  ngOnDestroy(): void {
    this.langChangeSub?.unsubscribe();
  }

  private buildMenuItems(): void {
    this.menuItems = [
      {
        label: this.t('admin.products.title'),
        icon: 'pi pi-box',
        routerLink: '/admin/products',
      },
      {
        label: this.t('admin.listings.title'),
        icon: 'pi pi-list',
        routerLink: '/admin/listings',
      },
      {
        label: this.t('admin.brands.title'),
        icon: 'pi pi-bookmark',
        routerLink: '/admin/brands',
      },
      {
        label: this.t('nav.myProfile'),
        icon: 'pi pi-user',
        routerLink: '/admin/profile',
      },
    ];
  }

  private t(key: string): string {
    return this.translate.instant(key);
  }
}
