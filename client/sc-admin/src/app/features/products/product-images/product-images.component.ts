import {
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  OnInit,
  computed,
  inject,
  signal,
} from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { FileUpload, FileUploadModule, FileUploadHandlerEvent } from 'primeng/fileupload';
import { ButtonModule } from 'primeng/button';
import { ImageModule } from 'primeng/image';
import { ToastModule } from 'primeng/toast';
import { ConfirmDialogModule } from 'primeng/confirmdialog';
import { ProgressSpinnerModule } from 'primeng/progressspinner';
import { ConfirmationService, MessageService } from 'primeng/api';
import { TranslateModule, TranslateService } from '@ngx-translate/core';
import { firstValueFrom } from 'rxjs';
import { ProductApiService } from '../../../core/services/product-api.service';
import { AdminProductDTO, ImageVariantUrls } from '../../../core/models/product.model';
import { StatusBadgeComponent } from '../../../shared/components/status-badge/status-badge.component';

@Component({
  selector: 'app-product-images',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [
    FileUploadModule,
    ButtonModule,
    ImageModule,
    ToastModule,
    ConfirmDialogModule,
    ProgressSpinnerModule,
    TranslateModule,
    StatusBadgeComponent,
  ],
  providers: [ConfirmationService, MessageService],
  templateUrl: './product-images.component.html',
  styleUrl: './product-images.component.scss',
})
export class ProductImagesComponent implements OnInit {
  protected readonly router = inject(Router);
  private readonly route = inject(ActivatedRoute);
  private readonly productApi = inject(ProductApiService);
  private readonly confirmationService = inject(ConfirmationService);
  private readonly messageService = inject(MessageService);
  private readonly translate = inject(TranslateService);
  private readonly cdr = inject(ChangeDetectorRef);

  protected readonly product = signal<AdminProductDTO | null>(null);
  protected readonly loading = signal(false);
  protected readonly refreshing = signal(false);
  protected readonly uploadingPreview = signal(false);
  protected readonly uploadingImages = signal(false);

  protected readonly galleryImages = computed(() => {
    const p = this.product();
    if (!p) return [] as ImageVariantUrls[];
    return p.image_urls;
  });

  private productId: string | null = null;

  ngOnInit(): void {
    this.productId = this.route.snapshot.paramMap.get('id');
    if (!this.productId) {
      return;
    }
    void this.loadProduct();
  }

  protected async onPreviewUpload(event: FileUploadHandlerEvent, upload: FileUpload): Promise<void> {
    if (!this.productId) return;
    const file = event.files[0];
    if (!file) return;

    this.uploadingPreview.set(true);
    try {
      const res = await firstValueFrom(this.productApi.uploadPreview(this.productId, file));
      if (res.success) {
        this.toastSuccess('admin.products.form.messages.previewQueuedSummary', 'admin.products.form.messages.previewQueuedDetail');
        await this.loadProduct();
      } else {
        this.toastError('admin.products.form.messages.previewUploadFailed', res.error);
      }
    } catch (err: any) {
      this.toastError('admin.products.form.messages.previewUploadFailed', err?.error?.error);
    } finally {
      this.uploadingPreview.set(false);
      upload.clear();
      this.cdr.markForCheck();
    }
  }

  protected async onImagesUpload(event: FileUploadHandlerEvent, upload: FileUpload): Promise<void> {
    if (!this.productId) return;
    const files = event.files as File[];
    if (!files?.length) return;

    this.uploadingImages.set(true);
    try {
      const res = await firstValueFrom(this.productApi.uploadImages(this.productId, files));
      if (res.success) {
        this.toastSuccess('admin.products.form.messages.imagesQueuedSummary', 'admin.products.form.messages.imagesQueuedDetail');
        await this.loadProduct();
      } else {
        this.toastError('admin.products.form.messages.imagesUploadFailed', res.error);
      }
    } catch (err: any) {
      this.toastError('admin.products.form.messages.imagesUploadFailed', err?.error?.error);
    } finally {
      this.uploadingImages.set(false);
      upload.clear();
      this.cdr.markForCheck();
    }
  }

  protected confirmDelete(): void {
    const product = this.product();
    if (!product) return;

    this.confirmationService.confirm({
      header: this.translate.instant('admin.products.list.confirmDeleteTitle'),
      message: this.translate.instant('admin.products.list.confirmDeleteMessage', { name: product.name }),
      icon: 'pi pi-exclamation-triangle',
      acceptButtonStyleClass: 'p-button-danger',
      accept: () => void this.deleteProduct(product.id, product.version),
    });
  }

  protected confirmImageDelete(url: ImageVariantUrls): void {
    this.confirmationService.confirm({
      header: this.translate.instant('admin.products.images.deleteImage'),
      message: this.translate.instant('admin.products.images.confirmDeleteImage'),
      icon: 'pi pi-exclamation-triangle',
      acceptButtonStyleClass: 'p-button-danger',
      accept: () => this.mockDeleteImage(url),
    });
  }

  protected async refreshGallery(): Promise<void> {
    if (this.refreshing()) return;
    this.refreshing.set(true);
    await this.loadProduct();
    this.refreshing.set(false);
    this.cdr.markForCheck();
  }

  private mockDeleteImage(_url: ImageVariantUrls): void {
    this.messageService.add({
      severity: 'info',
      summary: this.translate.instant('admin.products.images.mockNotImplemented.summary'),
      detail: this.translate.instant('admin.products.images.mockNotImplemented.detail'),
    });
  }

  private async loadProduct(): Promise<void> {
    if (!this.productId) return;
    this.loading.set(true);
    try {
      const res = await firstValueFrom(this.productApi.getProduct(this.productId));
      if (res.success && res.data) {
        this.product.set(res.data);
      } else {
        this.product.set(null);
      }
    } catch {
      this.product.set(null);
    } finally {
      this.loading.set(false);
      this.cdr.markForCheck();
    }
  }

  private async deleteProduct(id: string, expectedVersion: number): Promise<void> {
    const summary = this.translate.instant('admin.products.list.deleteFailedSummary');
    const fallback = this.translate.instant('admin.products.list.deleteFailedDetail');
    try {
      const res = await firstValueFrom(this.productApi.deleteProduct(id, expectedVersion));
      if (!res.success) {
        this.messageService.add({ severity: 'error', summary, detail: res.error ?? fallback });
        return;
      }
      this.router.navigate(['/admin/products']);
    } catch (error: any) {
      this.messageService.add({ severity: 'error', summary, detail: error?.error?.error ?? fallback });
    }
  }

  private toastSuccess(summaryKey: string, detailKey: string): void {
    this.messageService.add({
      severity: 'success',
      summary: this.translate.instant(summaryKey),
      detail: this.translate.instant(detailKey),
    });
  }

  private toastError(summaryKey: string, detailOverride?: string | null): void {
    this.messageService.add({
      severity: 'error',
      summary: this.translate.instant(summaryKey),
      detail: detailOverride ?? this.translate.instant('admin.products.form.messages.genericError'),
    });
  }
}
