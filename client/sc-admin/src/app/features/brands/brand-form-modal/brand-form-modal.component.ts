import {
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  OnDestroy,
  effect,
  inject,
  input,
  output,
  signal,
} from '@angular/core';
import { FormBuilder, FormGroup, ReactiveFormsModule, Validators } from '@angular/forms';
import { DialogModule } from 'primeng/dialog';
import { ButtonModule } from 'primeng/button';
import { InputTextModule } from 'primeng/inputtext';
import { SelectModule } from 'primeng/select';
import { ToastModule } from 'primeng/toast';
import { MessageService } from 'primeng/api';
import { TranslateModule, TranslateService } from '@ngx-translate/core';
import { Subscription, firstValueFrom } from 'rxjs';
import { BrandApiService } from '../../../core/services/brand-api.service';
import { Brand, CreateBrandRequest, UpdateBrandRequest } from '../../../core/models/brand.model';

@Component({
  selector: 'app-brand-form-modal',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [
    ReactiveFormsModule,
    DialogModule,
    ButtonModule,
    InputTextModule,
    SelectModule,
    ToastModule,
    TranslateModule,
  ],
  providers: [MessageService],
  templateUrl: './brand-form-modal.component.html',
  styleUrl: './brand-form-modal.component.scss',
})
export class BrandFormModalComponent implements OnDestroy {
  readonly visible = input.required<boolean>();
  readonly brandId = input<number | null>(null);
  readonly visibleChange = output<boolean>();
  readonly saved = output<Brand>();

  private readonly fb = inject(FormBuilder);
  private readonly brandApi = inject(BrandApiService);
  private readonly messageService = inject(MessageService);
  private readonly cdr = inject(ChangeDetectorRef);
  private readonly translate = inject(TranslateService);

  form!: FormGroup;
  existingBrand: Brand | null = null;

  protected readonly loadingBrand = signal(false);
  protected readonly saving = signal(false);

  tierOptions: Array<{ label: string; value: string }> = [];

  private readonly tierValues = ['mass', 'premium', 'luxury'] as const;

  private langChangeSub: Subscription | null = null;

  constructor() {
    this.buildForm();
    this.buildLocalizedOptions();
    this.langChangeSub = this.translate.onLangChange.subscribe(() => {
      this.buildLocalizedOptions();
      this.cdr.markForCheck();
    });

    effect(() => {
      const isVisible = this.visible();
      if (isVisible) {
        this.existingBrand = null;
        this.buildForm();

        const id = this.brandId();
        if (id) {
          this.loadBrand(id);
        }
      }
    });
  }

  ngOnDestroy(): void {
    this.langChangeSub?.unsubscribe();
  }

  private buildLocalizedOptions(): void {
    this.tierOptions = this.tierValues.map(value => ({
      label: this.t(`brandTiers.${value}`, value),
      value,
    }));
  }

  private buildForm(): void {
    this.form = this.fb.group({
      name: ['', [Validators.required, Validators.maxLength(120)]],
      code: ['', [Validators.required, Validators.maxLength(32)]],
      tier: ['mass', Validators.required],
      country: [''],
    });
  }

  private async loadBrand(id: number): Promise<void> {
    this.loadingBrand.set(true);
    try {
      const res = await firstValueFrom(this.brandApi.getById(id));
      if (res.success && res.data) {
        this.existingBrand = res.data;
        this.form.patchValue({
          name: res.data.name,
          code: res.data.code,
          tier: res.data.tier,
          country: res.data.country ?? '',
        });
        this.cdr.markForCheck();
      }
    } catch {
      this.messageService.add({
        severity: 'error',
        summary: this.t('admin.brands.form.messages.loadFailedSummary', this.t('common.error')),
        detail: this.t('admin.brands.form.messages.loadFailedDetail', 'Failed to load brand'),
      });
    } finally {
      this.loadingBrand.set(false);
    }
  }

  async onSubmit(): Promise<void> {
    this.form.markAllAsTouched();
    if (this.form.invalid) return;

    this.saving.set(true);
    try {
      const fv = this.form.getRawValue();
      const country = (fv.country as string).trim();
      const id = this.brandId();

      if (id) {
        const updateReq: UpdateBrandRequest = {
          name: fv.name,
          code: fv.code,
          tier: fv.tier,
          country: country || undefined,
        };
        const res = await firstValueFrom(this.brandApi.update(id, updateReq));
        if (!res.success || !res.data) {
          this.messageService.add({
            severity: 'error',
            summary: this.t('admin.brands.form.messages.updateFailedSummary', this.t('common.error')),
            detail: res.error ?? this.t('admin.brands.form.messages.updateFailedDetail', 'Failed to update brand'),
          });
          return;
        }
        this.saved.emit(res.data);
      } else {
        const createReq: CreateBrandRequest = {
          name: fv.name,
          code: fv.code,
          tier: fv.tier,
          country: country || undefined,
        };
        const res = await firstValueFrom(this.brandApi.create(createReq));
        if (!res.success || !res.data) {
          this.messageService.add({
            severity: 'error',
            summary: this.t('admin.brands.form.messages.createFailedSummary', this.t('common.error')),
            detail: res.error ?? this.t('admin.brands.form.messages.createFailedDetail', 'Failed to create brand'),
          });
          return;
        }
        this.saved.emit(res.data);
      }
    } catch (err: any) {
      const errMsg = err.error?.error ?? err.message ?? this.t('admin.brands.form.messages.genericError', 'An error occurred');
      this.messageService.add({
        severity: 'error',
        summary: this.t('admin.brands.form.messages.saveFailedSummary', this.t('common.error')),
        detail: errMsg,
      });
    } finally {
      this.saving.set(false);
    }
  }

  onCancel(): void {
    this.visibleChange.emit(false);
  }

  private t(key: string, fallback?: string, params?: Record<string, string | number>): string {
    const translated = this.translate.instant(key, params);
    return translated === key ? (fallback ?? key) : translated;
  }
}
