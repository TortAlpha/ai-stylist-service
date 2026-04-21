import { ChangeDetectionStrategy, Component, OnInit, inject, signal } from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { BrandFormModalComponent } from '../brand-form-modal/brand-form-modal.component';

@Component({
  selector: 'app-brand-form-page',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [BrandFormModalComponent],
  templateUrl: './brand-form-page.component.html',
})
export class BrandFormPageComponent implements OnInit {
  private readonly route = inject(ActivatedRoute);
  private readonly router = inject(Router);

  protected readonly visible = signal(true);
  protected readonly brandId = signal<number | null>(null);

  ngOnInit(): void {
    const id = this.route.snapshot.paramMap.get('id');
    this.brandId.set(id ? Number(id) : null);
  }

  onVisibleChange(next: boolean): void {
    this.visible.set(next);
    if (!next) {
      this.navigateAfterClose();
    }
  }

  onSaved(): void {
    this.navigateAfterClose();
  }

  private navigateAfterClose(): void {
    this.router.navigate(['/admin/brands']);
  }
}
