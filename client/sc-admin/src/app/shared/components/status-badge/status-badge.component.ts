import { Component, ChangeDetectionStrategy, input, computed } from '@angular/core';
import { TranslateModule } from '@ngx-translate/core';

export type ProductStatus =
  | 'intake' | 'inspection' | 'rejected' | 'preparation'
  | 'photo_queue' | 'photo_done' | 'ready' | 'reserved' | 'sold' | 'returned';

export type ListingStatus = 'active' | 'paused' | 'sold' | 'removed';

export type StatusType = 'product' | 'listing';

@Component({
  selector: 'app-status-badge',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [TranslateModule],
  templateUrl: './status-badge.component.html',
  styleUrl: './status-badge.component.scss',
})
export class StatusBadgeComponent {
  status = input.required<string>();
  type = input<StatusType>('product');

  badgeClass = computed(() => {
    const s = this.status();
    const t = this.type();
    return t === 'listing' ? `badge listing-${s}` : `badge status-${s}`;
  });

  translateKey = computed(() => {
    const s = this.status();
    const t = this.type();
    return t === 'listing' ? `listingStatuses.${s}` : `statuses.${s}`;
  });
}
