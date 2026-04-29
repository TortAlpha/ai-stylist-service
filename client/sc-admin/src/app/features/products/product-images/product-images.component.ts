import {
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  OnDestroy,
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
import { AdminProductDTO, ProductImageUrls } from '../../../core/models/product.model';
import { StatusBadgeComponent } from '../../../shared/components/status-badge/status-badge.component';

const UPLOAD_POLL_INTERVAL_MS = 1500;
const UPLOAD_POLL_TIMEOUT_MS = 60000;
const PREVIEW_REPLACE_MIN_WAIT_MS = 3000;

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
export class ProductImagesComponent implements OnInit, OnDestroy {
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
    if (!p) return [] as ProductImageUrls[];
    return p.image_urls;
  });

  protected readonly deletingImageIds = signal<Set<number>>(new Set());
  protected readonly deletingPreview = signal(false);

  private productId: string | null = null;
  private destroyed = false;

  ngOnInit(): void {
    this.productId = this.route.snapshot.paramMap.get('id');
    if (!this.productId) {
      return;
    }
    void this.loadProduct();
  }

  ngOnDestroy(): void {
    this.destroyed = true;
  }

  protected async onPreviewUpload(event: FileUploadHandlerEvent, upload: FileUpload): Promise<void> {
    if (!this.productId) return;
    const file = event.files[0];
    if (!file) return;

    const previousPreviewUrl = this.product()?.preview_url?.full ?? null;
    this.uploadingPreview.set(true);
    try {
      const res = await firstValueFrom(this.productApi.uploadPreview(this.productId, file));
      if (res.success) {
        this.toastSuccess(
          'admin.products.form.messages.previewQueuedSummary',
          'admin.products.form.messages.previewQueuedDetail',
        );
        const ready = await this.pollProductUntil(
          product => this.hasPreviewUploadCompleted(product, previousPreviewUrl),
          previousPreviewUrl ? PREVIEW_REPLACE_MIN_WAIT_MS : 0,
        );
        if (ready) {
          this.toastSuccess(
            'admin.products.form.messages.previewReadySummary',
            'admin.products.form.messages.previewReadyDetail',
          );
        } else if (!this.destroyed) {
          this.toastWarn(
            'admin.products.form.messages.processingTimeoutSummary',
            'admin.products.form.messages.previewProcessingTimeoutDetail',
          );
        }
        return;
      }
      this.toastError('admin.products.form.messages.previewUploadFailed', res.error);
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

    const previousImageIds = new Set(this.galleryImages().map(image => image.id));
    const expectedImageCount = previousImageIds.size + files.length;
    this.uploadingImages.set(true);
    try {
      const res = await firstValueFrom(this.productApi.uploadImages(this.productId, files));
      if (res.success) {
        this.toastSuccess(
          'admin.products.form.messages.imagesQueuedSummary',
          'admin.products.form.messages.imagesQueuedDetail',
        );
        const ready = await this.pollProductUntil(product => {
          const newImageCount = product.image_urls.filter(
            image => !previousImageIds.has(image.id),
          ).length;
          return newImageCount >= files.length || product.image_urls.length >= expectedImageCount;
        });
        if (ready) {
          this.toastSuccess(
            'admin.products.form.messages.imagesReadySummary',
            'admin.products.form.messages.imagesReadyDetail',
          );
        } else if (!this.destroyed) {
          this.toastWarn(
            'admin.products.form.messages.processingTimeoutSummary',
            'admin.products.form.messages.imagesProcessingTimeoutDetail',
          );
        }
        return;
      }
      this.toastError('admin.products.form.messages.imagesUploadFailed', res.error);
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

  protected confirmImageDelete(image: ProductImageUrls): void {
    this.confirmationService.confirm({
      header: this.translate.instant('admin.products.images.deleteImage'),
      message: this.translate.instant('admin.products.images.confirmDeleteImage'),
      icon: 'pi pi-exclamation-triangle',
      acceptButtonStyleClass: 'p-button-danger',
      accept: () => void this.deleteImage(image),
    });
  }

  protected confirmPreviewDelete(): void {
    if (!this.product()?.preview_url) return;
    this.confirmationService.confirm({
      header: this.translate.instant('admin.products.images.deletePreview'),
      message: this.translate.instant('admin.products.images.confirmDeletePreview'),
      icon: 'pi pi-exclamation-triangle',
      acceptButtonStyleClass: 'p-button-danger',
      accept: () => void this.deletePreview(),
    });
  }

  protected isDeletingImage(image: ProductImageUrls): boolean {
    return this.deletingImageIds().has(image.id);
  }

  protected async refreshGallery(): Promise<void> {
    if (this.refreshing()) return;
    this.refreshing.set(true);
    await this.loadProduct();
    this.refreshing.set(false);
    this.cdr.markForCheck();
  }

  private async deleteImage(image: ProductImageUrls): Promise<void> {
    if (!this.productId) return;
    if (this.deletingImageIds().has(image.id)) return;
    this.markImageDeleting(image.id, true);
    try {
      const res = await firstValueFrom(this.productApi.deleteImage(this.productId, image.id));
      if (res.success) {
        this.toastSuccess('admin.products.images.imageDeleted.summary', 'admin.products.images.imageDeleted.detail');
        await this.loadProduct();
      } else {
        this.toastError('admin.products.images.deleteFailed.summary', res.error);
      }
    } catch (err: any) {
      this.toastError('admin.products.images.deleteFailed.summary', err?.error?.error);
    } finally {
      this.markImageDeleting(image.id, false);
      this.cdr.markForCheck();
    }
  }

  private async deletePreview(): Promise<void> {
    if (!this.productId || this.deletingPreview()) return;
    this.deletingPreview.set(true);
    try {
      const res = await firstValueFrom(this.productApi.deletePreview(this.productId));
      if (res.success) {
        this.toastSuccess('admin.products.images.previewDeleted.summary', 'admin.products.images.previewDeleted.detail');
        await this.loadProduct();
      } else {
        this.toastError('admin.products.images.deleteFailed.summary', res.error);
      }
    } catch (err: any) {
      this.toastError('admin.products.images.deleteFailed.summary', err?.error?.error);
    } finally {
      this.deletingPreview.set(false);
      this.cdr.markForCheck();
    }
  }

  private markImageDeleting(id: number, deleting: boolean): void {
    const next = new Set(this.deletingImageIds());
    if (deleting) {
      next.add(id);
    } else {
      next.delete(id);
    }
    this.deletingImageIds.set(next);
  }

  private async loadProduct(): Promise<void> {
    if (!this.productId) return;
    this.loading.set(true);
    try {
      this.product.set(await this.fetchProduct());
    } finally {
      this.loading.set(false);
      this.cdr.markForCheck();
    }
  }

  private async fetchProduct(): Promise<AdminProductDTO | null> {
    if (!this.productId) return null;
    try {
      const res = await firstValueFrom(this.productApi.getProduct(this.productId));
      return res.success && res.data ? res.data : null;
    } catch {
      return null;
    }
  }

  private async pollProductUntil(
    isReady: (product: AdminProductDTO) => boolean,
    minWaitMs = 0,
  ): Promise<boolean> {
    const startedAt = Date.now();

    while (!this.destroyed && Date.now() - startedAt < UPLOAD_POLL_TIMEOUT_MS) {
      await this.delay(UPLOAD_POLL_INTERVAL_MS);
      if (this.destroyed) return false;

      const next = await this.fetchProduct();
      if (!next) continue;

      this.product.set(next);
      this.cdr.markForCheck();

      const elapsed = Date.now() - startedAt;
      if (elapsed >= minWaitMs && isReady(next)) {
        return true;
      }
    }

    return false;
  }

  private hasPreviewUploadCompleted(
    product: AdminProductDTO,
    previousPreviewUrl: string | null,
  ): boolean {
    if (!product.preview_url) return false;
    if (!previousPreviewUrl) return true;
    return product.preview_url.full !== previousPreviewUrl;
  }

  private delay(ms: number): Promise<void> {
    return new Promise(resolve => window.setTimeout(resolve, ms));
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

  private toastWarn(summaryKey: string, detailKey: string): void {
    this.messageService.add({
      severity: 'warn',
      summary: this.translate.instant(summaryKey),
      detail: this.translate.instant(detailKey),
    });
  }
}
