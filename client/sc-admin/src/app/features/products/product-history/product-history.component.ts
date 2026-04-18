import {
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  inject,
  OnInit,
  signal,
} from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { ButtonModule } from 'primeng/button';
import { firstValueFrom } from 'rxjs';
import { ProductApiService } from '../../../core/services/product-api.service';
import { AdminProductDTO } from '../../../core/models/product.model';

@Component({
  selector: 'app-product-history',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [ButtonModule],
  templateUrl: './product-history.component.html',
  styleUrl: './product-history.component.scss',
})
export class ProductHistoryComponent implements OnInit {
  protected readonly router = inject(Router);
  private readonly route = inject(ActivatedRoute);
  private readonly productApi = inject(ProductApiService);
  private readonly cdr = inject(ChangeDetectorRef);

  protected readonly product = signal<AdminProductDTO | null>(null);
  private productId!: string;

  ngOnInit(): void {
    this.productId = this.route.snapshot.paramMap.get('id')!;
    this.loadProduct();
  }

  private async loadProduct(): Promise<void> {
    try {
      const res = await firstValueFrom(this.productApi.getProduct(this.productId));
      if (res.success && res.data) {
        this.product.set(res.data);
        this.cdr.markForCheck();
      }
    } catch {
      // Non-critical
    }
  }
}
