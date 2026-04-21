import { Component, EventEmitter, inject, Input, OnChanges, Output, SimpleChanges } from '@angular/core';
import { FormBuilder, FormGroup, ReactiveFormsModule, Validators } from '@angular/forms';
import { DialogModule } from 'primeng/dialog';
import { InputTextModule } from 'primeng/inputtext';
import { SelectModule } from 'primeng/select';
import { ButtonModule } from 'primeng/button';
import { TranslateModule } from '@ngx-translate/core';
import { ListingStore } from '../../../store/listing.store';
import { Marketplace, ProductListing } from '../../../core/models/listing.model';

@Component({
  selector: 'app-listing-form',
  standalone: true,
  imports: [ReactiveFormsModule, DialogModule, InputTextModule, SelectModule, ButtonModule, TranslateModule],
  providers: [ListingStore],
  templateUrl: './listing-form.component.html',
  styleUrl: './listing-form.component.scss',
})
export class ListingFormComponent implements OnChanges {
  @Input() visible = false;
  @Input() productId!: string;
  @Input() listing: ProductListing | null = null;
  @Input() marketplaces: Marketplace[] = [];
  @Output() saved = new EventEmitter<void>();
  @Output() cancelled = new EventEmitter<void>();

  private readonly fb = inject(FormBuilder);
  private readonly listingStore = inject(ListingStore);

  form!: FormGroup;
  saving = false;

  currencies = ['USD', 'EUR', 'GBP', 'UAH'];

  constructor() {
    this.form = this.fb.group({
      marketplace_id: [null, Validators.required],
      external_id: [''],
      external_url: [''],
      listing_price: [''],
      currency: ['USD'],
    });
  }

  ngOnChanges(changes: SimpleChanges): void {
    if (changes['visible'] && this.visible) {
      if (this.listing) {
        this.form.patchValue({
          marketplace_id: this.listing.marketplace_id,
          external_id: this.listing.external_id ?? '',
          external_url: this.listing.external_url ?? '',
          listing_price: this.listing.listing_price ?? '',
          currency: this.listing.currency ?? 'USD',
        });
      } else {
        this.form.reset({ currency: 'USD' });
      }
    }
  }

  async onSubmit(): Promise<void> {
    if (this.form.invalid) return;
    this.saving = true;

    const formValue = this.form.getRawValue();

    const result = await this.listingStore.createListing({
      product_id: this.productId,
      marketplace_id: formValue.marketplace_id,
      external_id: formValue.external_id || undefined,
      external_url: formValue.external_url || undefined,
      listing_price: formValue.listing_price || undefined,
      currency: formValue.currency || undefined,
    });

    if (result) {
      this.saved.emit();
    }

    this.saving = false;
  }

  cancel(): void {
    this.cancelled.emit();
  }

  onVisibleChange(visible: boolean): void {
    if (!visible) {
      this.cancelled.emit();
    }
  }
}
