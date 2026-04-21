import {
  ChangeDetectionStrategy,
  Component,
  inject,
  OnInit,
} from '@angular/core';
import { Router } from '@angular/router';
import { DatePipe, SlicePipe } from '@angular/common';
import { TableModule } from 'primeng/table';
import { ButtonModule } from 'primeng/button';
import { TagModule } from 'primeng/tag';
import { TranslateModule } from '@ngx-translate/core';
import { ListingStore } from '../../../store/listing.store';
import { ListingWithProduct, Marketplace } from '../../../core/models/listing.model';

@Component({
  selector: 'app-my-listings',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [TableModule, ButtonModule, TagModule, DatePipe, SlicePipe, TranslateModule],
  templateUrl: './my-listings.component.html',
  styleUrl: './my-listings.component.scss',
})
export class MyListingsComponent implements OnInit {
  protected readonly store = inject(ListingStore);
  private readonly router = inject(Router);

  ngOnInit(): void {
    this.store.loadMarketplaces();
    this.store.loadAllListings();
  }

  getMarketplaceName(id: number): string {
    const mp = this.store.marketplaces().find((m: Marketplace) => m.id === id);
    return mp?.name ?? `Marketplace #${id}`;
  }

  goToProductListings(productId: string): void {
    this.router.navigate(['/admin/products', productId, 'listings']);
  }

  getStatusLabel(status: string): string {
    const map: Record<string, string> = {
      active: 'Active',
      paused: 'Paused',
      sold: 'Sold',
      removed: 'Removed',
    };
    return map[status] ?? status;
  }

  getStatusSeverity(status: string): 'success' | 'secondary' | 'info' | 'warn' | 'danger' | 'contrast' {
    const map: Record<string, 'success' | 'secondary' | 'info' | 'warn' | 'danger' | 'contrast'> = {
      active: 'success',
      paused: 'warn',
      sold: 'contrast',
      removed: 'danger',
    };
    return map[status] ?? 'secondary';
  }
}
