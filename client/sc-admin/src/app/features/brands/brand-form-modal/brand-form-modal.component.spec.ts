import { ComponentRef } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { provideNoopAnimations } from '@angular/platform-browser/animations';
import { TranslateModule } from '@ngx-translate/core';
import { of, throwError } from 'rxjs';
import { describe, expect, it, beforeEach, vi } from 'vitest';
import { BrandFormModalComponent } from './brand-form-modal.component';
import { BrandApiService } from '../../../core/services/brand-api.service';

const NOW = '2026-01-01T00:00:00.000Z';

function createBrand(overrides: Partial<{ id: number; name: string; code: string }> = {}) {
  return {
    id: overrides.id ?? 1,
    name: overrides.name ?? 'Acme',
    code: overrides.code ?? 'ACME',
    tier: 'premium' as const,
    country: null,
    created_at: NOW,
  };
}

describe('BrandFormModalComponent', () => {
  let fixture: ComponentFixture<BrandFormModalComponent>;
  let ref: ComponentRef<BrandFormModalComponent>;
  let component: BrandFormModalComponent;
  let api: { create: ReturnType<typeof vi.fn>; update: ReturnType<typeof vi.fn>; getById: ReturnType<typeof vi.fn> };

  beforeEach(() => {
    api = {
      create: vi.fn().mockReturnValue(of({ success: true, data: createBrand({ id: 99, name: 'New', code: 'NEW' }), error: null })),
      update: vi.fn().mockReturnValue(of({ success: true, data: createBrand(), error: null })),
      getById: vi.fn().mockReturnValue(of({ success: true, data: createBrand(), error: null })),
    };

    TestBed.configureTestingModule({
      imports: [BrandFormModalComponent, TranslateModule.forRoot()],
      providers: [
        provideNoopAnimations(),
        { provide: BrandApiService, useValue: api },
      ],
    });

    fixture = TestBed.createComponent(BrandFormModalComponent);
    ref = fixture.componentRef;
    component = fixture.componentInstance;
    ref.setInput('visible', false);
  });

  it('form is invalid when required fields empty', () => {
    component.form.patchValue({ name: '', code: '', tier: null });
    expect(component.form.valid).toBe(false);
  });

  it('form is valid with name, code and tier', () => {
    component.form.patchValue({ name: 'Gucci', code: 'GUCCI', tier: 'luxury' });
    expect(component.form.valid).toBe(true);
  });

  it('emits created brand on successful create', async () => {
    const savedSpy = vi.fn();
    component.saved.subscribe(savedSpy);
    component.form.patchValue({ name: 'New', code: 'NEW', tier: 'mass' });

    await component.onSubmit();

    expect(api.create).toHaveBeenCalledWith({ name: 'New', code: 'NEW', tier: 'mass', country: undefined });
    expect(savedSpy).toHaveBeenCalledTimes(1);
    const emitted = savedSpy.mock.calls[0][0];
    expect(emitted.id).toBe(99);
  });

  it('trims country whitespace before submit', async () => {
    component.form.patchValue({ name: 'A', code: 'A', tier: 'mass', country: '  Italy  ' });
    await component.onSubmit();
    expect(api.create).toHaveBeenCalledWith({ name: 'A', code: 'A', tier: 'mass', country: 'Italy' });
  });

  it('calls update endpoint when brandId is provided', async () => {
    ref.setInput('brandId', 7);
    component.form.patchValue({ name: 'Updated', code: 'U', tier: 'mass' });

    await component.onSubmit();

    expect(api.update).toHaveBeenCalledTimes(1);
    expect(api.update.mock.calls[0][0]).toBe(7);
  });

  it('does not emit saved on API error response', async () => {
    api.create.mockReturnValue(of({ success: false, data: null, error: 'taken' }));
    const savedSpy = vi.fn();
    component.saved.subscribe(savedSpy);
    component.form.patchValue({ name: 'N', code: 'N', tier: 'mass' });

    await component.onSubmit();

    expect(savedSpy).not.toHaveBeenCalled();
  });

  it('swallows thrown errors without emitting saved', async () => {
    api.create.mockReturnValue(throwError(() => ({ status: 500, message: 'boom' })));
    const savedSpy = vi.fn();
    component.saved.subscribe(savedSpy);
    component.form.patchValue({ name: 'N', code: 'N', tier: 'mass' });

    await component.onSubmit();

    expect(savedSpy).not.toHaveBeenCalled();
  });

  it('onCancel emits visibleChange false', () => {
    const spy = vi.fn();
    component.visibleChange.subscribe(spy);
    component.onCancel();
    expect(spy).toHaveBeenCalledWith(false);
  });
});
