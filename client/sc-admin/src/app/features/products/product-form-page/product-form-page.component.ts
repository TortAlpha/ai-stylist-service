import { ChangeDetectionStrategy, Component, OnInit, inject, signal } from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { ProductFormModalComponent } from '../product-form-modal/product-form-modal.component';

@Component({
  selector: 'app-product-form-page',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [ProductFormModalComponent],
  templateUrl: './product-form-page.component.html',
})
export class ProductFormPageComponent implements OnInit {
  private readonly route = inject(ActivatedRoute);
  private readonly router = inject(Router);

  protected readonly visible = signal(true);
  protected readonly productId = signal<string | null>(null);

  ngOnInit(): void {
    this.productId.set(this.route.snapshot.paramMap.get('id'));
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
    const productId = this.productId();
    if (productId) {
      this.router.navigate(['/admin/products', productId]);
      return;
    }

    this.router.navigate(['/admin/products']);
  }
}
