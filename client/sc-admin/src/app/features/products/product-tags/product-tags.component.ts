import { Component, inject, OnInit } from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { FormsModule } from '@angular/forms';
import { MultiSelectModule } from 'primeng/multiselect';
import { ButtonModule } from 'primeng/button';
import { CardModule } from 'primeng/card';
import { firstValueFrom } from 'rxjs';
import { ProductApiService } from '../../../core/services/product-api.service';
import { TagStore } from '../../../store/tag.store';
import { AdminProductDTO } from '../../../core/models/product.model';

@Component({
  selector: 'app-product-tags',
  standalone: true,
  imports: [FormsModule, MultiSelectModule, ButtonModule, CardModule],
  providers: [TagStore],
  templateUrl: './product-tags.component.html',
  styleUrl: './product-tags.component.scss',
})
export class ProductTagsComponent implements OnInit {
  protected readonly router = inject(Router);
  private readonly route = inject(ActivatedRoute);
  private readonly productApi = inject(ProductApiService);
  protected readonly tagStore = inject(TagStore);

  productId!: string;
  product: AdminProductDTO | null = null;
  selectedStyleIds: number[] = [];
  selectedVibeIds: number[] = [];
  selectedSeasonIds: number[] = [];
  saving = false;
  message: string | null = null;
  error: string | null = null;

  ngOnInit(): void {
    this.productId = this.route.snapshot.paramMap.get('id')!;
    this.tagStore.loadAll();
    this.loadProduct();
  }

  private async loadProduct(): Promise<void> {
    try {
      const res = await firstValueFrom(this.productApi.getProduct(this.productId));
      if (res.success && res.data) {
        this.product = res.data;
        // Tags in response are string names, not IDs — match against loaded tags
        // This will be populated once tags are loaded
      }
    } catch {
      this.error = 'Failed to load product';
    }
  }

  async saveAll(): Promise<void> {
    if (!this.product) return;
    this.saving = true;
    this.message = null;
    this.error = null;
    try {
      const res = await firstValueFrom(this.productApi.updateProduct(this.productId, {
        style_tag_ids: this.selectedStyleIds,
        vibe_tag_ids: this.selectedVibeIds,
        season_ids: this.selectedSeasonIds,
        expected_version: this.product.version,
      }));
      if (res.success) {
        this.message = 'Tags saved successfully.';
        if (res.data) {
          this.product = res.data;
        }
      } else {
        this.error = res.error ?? 'Failed to save tags';
      }
    } catch (err: any) {
      this.error = err.error?.error ?? 'Failed to save tags';
    } finally {
      this.saving = false;
    }
  }
}
