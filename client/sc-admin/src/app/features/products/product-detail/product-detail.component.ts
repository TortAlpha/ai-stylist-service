import {
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  OnDestroy,
  OnInit,
  inject,
  signal,
} from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { DatePipe } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ButtonModule } from 'primeng/button';
import { CardModule } from 'primeng/card';
import { ConfirmDialogModule } from 'primeng/confirmdialog';
import { ImageModule } from 'primeng/image';
import { SelectModule } from 'primeng/select';
import { ToastModule } from 'primeng/toast';
import { ConfirmationService, MessageService } from 'primeng/api';
import { TranslateModule, TranslateService } from '@ngx-translate/core';
import { Subscription, firstValueFrom } from 'rxjs';
import { ProductApiService } from '../../../core/services/product-api.service';
import {
  AdminProductDTO,
  ProductStatus,
  UpdateProductRequest,
} from '../../../core/models/product.model';
import { StatusBadgeComponent } from '../../../shared/components/status-badge/status-badge.component';

@Component({
  selector: 'app-product-detail',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [
    DatePipe,
    FormsModule,
    ButtonModule,
    CardModule,
    ConfirmDialogModule,
    ImageModule,
    SelectModule,
    ToastModule,
    TranslateModule,
    StatusBadgeComponent,
  ],
  providers: [ConfirmationService, MessageService],
  templateUrl: './product-detail.component.html',
  styleUrl: './product-detail.component.scss',
})
export class ProductDetailComponent implements OnInit, OnDestroy {
  protected readonly router = inject(Router);
  private readonly route = inject(ActivatedRoute);
  private readonly productApi = inject(ProductApiService);
  private readonly confirmationService = inject(ConfirmationService);
  private readonly messageService = inject(MessageService);
  private readonly translate = inject(TranslateService);
  private readonly cdr = inject(ChangeDetectorRef);

  protected readonly product = signal<AdminProductDTO | null>(null);
  protected readonly loading = signal(false);
  protected readonly statusSaving = signal(false);
  protected selectedStatus: ProductStatus | null = null;
  protected statusOptions: Array<{ label: string; value: ProductStatus }> = [];

  private productId: string | null = null;
  private langChangeSub: Subscription | null = null;
  private readonly statusValues: readonly ProductStatus[] = [
    'intake',
    'inspection',
    'rejected',
    'preparation',
    'photo_queue',
    'photo_done',
    'ready',
    'reserved',
    'sold',
    'returned',
  ] as const;

  ngOnInit(): void {
    this.buildStatusOptions();
    this.langChangeSub = this.translate.onLangChange.subscribe(() => {
      this.buildStatusOptions();
      this.cdr.markForCheck();
    });

    this.productId = this.route.snapshot.paramMap.get('id');
    if (!this.productId) {
      return;
    }
    void this.loadProduct(this.productId);
  }

  ngOnDestroy(): void {
    this.langChangeSub?.unsubscribe();
  }

  protected get canUpdateStatus(): boolean {
    const product = this.product();
    return !!product && !!this.selectedStatus && this.selectedStatus !== product.status && !this.statusSaving();
  }

  confirmDelete(): void {
    const product = this.product();
    if (!product) {
      return;
    }

    this.confirmationService.confirm({
      header: this.translate.instant('admin.products.list.confirmDeleteTitle'),
      message: this.translate.instant('admin.products.list.confirmDeleteMessage', { name: product.name }),
      icon: 'pi pi-exclamation-triangle',
      acceptButtonStyleClass: 'p-button-danger',
      accept: () => void this.deleteProduct(product.id, product.version),
    });
  }

  private async loadProduct(id: string): Promise<void> {
    this.loading.set(true);
    try {
      const res = await firstValueFrom(this.productApi.getProduct(id));
      if (res.success && res.data) {
        this.product.set(res.data);
        this.selectedStatus = this.toProductStatus(res.data.status);
      } else {
        this.product.set(null);
        this.selectedStatus = null;
      }
    } catch {
      this.product.set(null);
      this.selectedStatus = null;
    } finally {
      this.loading.set(false);
      this.cdr.markForCheck();
    }
  }

  protected async updateStatus(): Promise<void> {
    const product = this.product();
    const nextStatus = this.selectedStatus;
    if (!product || !nextStatus || nextStatus === product.status || this.statusSaving()) {
      return;
    }

    const failSummary = this.translate.instant('admin.products.form.messages.updateFailedSummary');
    const failDetail = this.translate.instant('admin.products.form.messages.updateFailedDetail');

    this.statusSaving.set(true);
    try {
      const request: UpdateProductRequest = {
        status: nextStatus,
        expected_version: product.version,
      };
      const res = await firstValueFrom(this.productApi.updateProduct(product.id, request));
      if (!res.success) {
        const detail = res.error ?? failDetail;
        const isConflict = this.isVersionConflictError(detail);
        this.messageService.add({
          severity: 'error',
          summary: failSummary,
          detail: isConflict ? this.translate.instant('admin.products.staleVersion') : detail,
        });
        if (isConflict) {
          await this.loadProduct(product.id);
        }
        return;
      }

      if (res.data) {
        this.product.set(res.data);
        this.selectedStatus = this.toProductStatus(res.data.status);
      } else {
        await this.loadProduct(product.id);
      }

      this.messageService.add({
        severity: 'success',
        summary: this.translate.instant('common.success'),
        detail: this.translate.instant('admin.products.detail.statusUpdated'),
      });
    } catch (error: any) {
      const detail = error?.error?.error ?? failDetail;
      const isConflict = error?.status === 409 || this.isVersionConflictError(detail);
      this.messageService.add({
        severity: 'error',
        summary: failSummary,
        detail: isConflict ? this.translate.instant('admin.products.staleVersion') : detail,
      });
      if (isConflict) {
        await this.loadProduct(product.id);
      }
    } finally {
      this.statusSaving.set(false);
      this.cdr.markForCheck();
    }
  }

  private async deleteProduct(id: string, expectedVersion: number): Promise<void> {
    const summary = this.translate.instant('admin.products.list.deleteFailedSummary');
    const fallbackDetail = this.translate.instant('admin.products.list.deleteFailedDetail');
    try {
      const res = await firstValueFrom(this.productApi.deleteProduct(id, expectedVersion));
      if (!res.success) {
        this.messageService.add({
          severity: 'error',
          summary,
          detail: res.error ?? fallbackDetail,
        });
        return;
      }
      this.router.navigate(['/admin/products']);
    } catch (error: any) {
      const detail = error?.error?.error ?? fallbackDetail;
      this.messageService.add({ severity: 'error', summary, detail });
    }
  }

  protected typeField(typeDetails: AdminProductDTO['type_details'], field: string): string | number | null {
    const value = (typeDetails as Record<string, unknown>)[field];
    if (value === undefined || value === null || value === '') {
      return null;
    }
    return value as string | number;
  }

  protected translateKey(key: string, fallback?: string | number | null): string {
    const translated = this.translate.instant(key);
    if (translated === key && fallback !== undefined && fallback !== null) {
      return String(fallback);
    }
    return translated;
  }

  private buildStatusOptions(): void {
    this.statusOptions = this.statusValues.map(value => ({
      label: this.translate.instant(`statuses.${value}`),
      value,
    }));
  }

  private isVersionConflictError(message: string): boolean {
    const normalized = message.toLowerCase();
    return normalized.includes('version') || normalized.includes('conflict');
  }

  private toProductStatus(value: string | null | undefined): ProductStatus | null {
    if (!value) {
      return null;
    }
    return this.statusValues.includes(value as ProductStatus) ? (value as ProductStatus) : null;
  }
}
