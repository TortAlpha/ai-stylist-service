import {
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  inject,
  OnInit,
  signal,
} from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { DatePipe } from '@angular/common';
import { FormBuilder, FormGroup, ReactiveFormsModule, Validators } from '@angular/forms';
import { TableModule } from 'primeng/table';
import { ButtonModule } from 'primeng/button';
import { TagModule } from 'primeng/tag';
import { ConfirmDialogModule } from 'primeng/confirmdialog';
import { DialogModule } from 'primeng/dialog';
import { InputTextModule } from 'primeng/inputtext';
import { SelectModule } from 'primeng/select';
import { ToastModule } from 'primeng/toast';
import { ConfirmationService, MessageService } from 'primeng/api';
import { TranslateModule } from '@ngx-translate/core';
import { firstValueFrom } from 'rxjs';
import { ListingStore } from '../../../store/listing.store';
import { ProductApiService } from '../../../core/services/product-api.service';
import { ProductListing, Marketplace } from '../../../core/models/listing.model';
import { AdminProductDTO } from '../../../core/models/product.model';

@Component({
  selector: 'app-product-listings',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [
    ReactiveFormsModule,
    TableModule,
    ButtonModule,
    TagModule,
    ConfirmDialogModule,
    DialogModule,
    InputTextModule,
    SelectModule,
    ToastModule,
    DatePipe,
    TranslateModule,
  ],
  providers: [ConfirmationService, MessageService],
  templateUrl: './product-listings.component.html',
  styleUrl: './product-listings.component.scss',
})
export class ProductListingsComponent implements OnInit {
  protected readonly store = inject(ListingStore);
  protected readonly router = inject(Router);
  private readonly route = inject(ActivatedRoute);
  private readonly productApi = inject(ProductApiService);
  private readonly confirmationService = inject(ConfirmationService);
  private readonly messageService = inject(MessageService);
  private readonly fb = inject(FormBuilder);
  private readonly cdr = inject(ChangeDetectorRef);

  protected readonly modalVisible = signal(false);
  protected readonly editingListing = signal<ProductListing | null>(null);
  protected readonly saving = signal(false);
  protected readonly product = signal<AdminProductDTO | null>(null);

  productId!: string;
  form!: FormGroup;

  currencies = ['EUR', 'USD', 'GBP', 'UAH'];

  statusOptions = [
    { label: 'Active', value: 'active' },
    { label: 'Paused', value: 'paused' },
    { label: 'Sold', value: 'sold' },
    { label: 'Removed', value: 'removed' },
  ];

  ngOnInit(): void {
    this.productId = this.route.snapshot.paramMap.get('id')!;
    this.buildForm();
    this.store.loadMarketplaces();
    this.store.loadProductListings(this.productId);
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

  private buildForm(): void {
    this.form = this.fb.group({
      marketplace_id: [null, Validators.required],
      external_id: [''],
      external_url: [''],
      listing_price: [''],
      currency: ['EUR'],
      status: ['active'],
      sold_price: [''],
    });
  }

  openAddModal(): void {
    this.editingListing.set(null);
    this.form.reset({ currency: 'EUR', status: 'active' });
    this.modalVisible.set(true);
  }

  openEditModal(listing: ProductListing): void {
    this.editingListing.set(listing);
    this.form.patchValue({
      marketplace_id: listing.marketplace_id,
      external_id: listing.external_id ?? '',
      external_url: listing.external_url ?? '',
      listing_price: listing.listing_price ?? '',
      currency: listing.currency ?? 'EUR',
      status: listing.status,
      sold_price: listing.sold_price ?? '',
    });
    this.modalVisible.set(true);
  }

  closeModal(): void {
    this.modalVisible.set(false);
    this.editingListing.set(null);
  }

  onModalVisibleChange(visible: boolean): void {
    if (!visible) this.closeModal();
  }

  async onSubmit(): Promise<void> {
    this.form.markAllAsTouched();
    if (this.form.invalid) return;

    this.saving.set(true);
    const fv = this.form.getRawValue();

    try {
      if (this.editingListing()) {
        const updateReq = {
          status: fv.status,
          sold_price: fv.sold_price || undefined,
        };
        const ok = await this.store.updateListing(this.editingListing()!.id, updateReq);
        if (ok) {
          this.messageService.add({ severity: 'success', summary: 'Saved', detail: 'Listing updated', life: 3000 });
          this.closeModal();
        } else {
          this.messageService.add({ severity: 'error', summary: 'Error', detail: this.store.error() ?? 'Failed to update' });
        }
      } else {
        const createReq = {
          product_id: this.productId,
          marketplace_id: fv.marketplace_id,
          external_id: fv.external_id || undefined,
          external_url: fv.external_url || undefined,
          listing_price: fv.listing_price || undefined,
          currency: fv.currency || undefined,
        };
        const result = await this.store.createListing(createReq);
        if (result) {
          this.messageService.add({ severity: 'success', summary: 'Created', detail: 'Listing added', life: 3000 });
          this.closeModal();
        } else {
          this.messageService.add({ severity: 'error', summary: 'Error', detail: this.store.error() ?? 'Failed to create' });
        }
      }
    } finally {
      this.saving.set(false);
    }
  }

  confirmDelete(listing: ProductListing): void {
    this.confirmationService.confirm({
      message: `Delete this ${this.getMarketplaceName(listing.marketplace_id)} listing?`,
      header: 'Delete Listing',
      icon: 'pi pi-exclamation-triangle',
      acceptButtonStyleClass: 'p-button-danger',
      accept: async () => {
        const ok = await this.store.deleteListing(listing.id);
        if (ok) {
          this.messageService.add({ severity: 'success', summary: 'Deleted', detail: 'Listing removed', life: 3000 });
        } else {
          this.messageService.add({ severity: 'error', summary: 'Error', detail: this.store.error() ?? 'Failed to delete' });
        }
      },
    });
  }

  getMarketplaceName(id: number): string {
    const mp = this.store.marketplaces().find((m: Marketplace) => m.id === id);
    return mp?.name ?? `Marketplace #${id}`;
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
