import {
  ChangeDetectionStrategy,
  Component,
  inject,
  OnInit,
  signal,
} from '@angular/core';
import { FormBuilder, FormGroup, ReactiveFormsModule, Validators } from '@angular/forms';
import { DatePipe } from '@angular/common';
import { CardModule } from 'primeng/card';
import { InputTextModule } from 'primeng/inputtext';
import { ButtonModule } from 'primeng/button';
import { DialogModule } from 'primeng/dialog';
import { DividerModule } from 'primeng/divider';
import { ToastModule } from 'primeng/toast';
import { MessageService } from 'primeng/api';
import { TranslateModule } from '@ngx-translate/core';
import { UserProfileStore } from '../../../store/user-profile.store';
import { AddressResponse } from '../../../core/models/user.model';

@Component({
  selector: 'app-my-profile',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [
    ReactiveFormsModule,
    DatePipe,
    CardModule,
    InputTextModule,
    ButtonModule,
    DialogModule,
    DividerModule,
    ToastModule,
    TranslateModule,
  ],
  providers: [MessageService],
  templateUrl: './my-profile.component.html',
  styleUrl: './my-profile.component.scss',
})
export class MyProfileComponent implements OnInit {
  protected readonly store = inject(UserProfileStore);
  private readonly fb = inject(FormBuilder);
  private readonly messageService = inject(MessageService);

  protected readonly addressModalVisible = signal(false);
  protected readonly editingAddress = signal<AddressResponse | null>(null);

  profileForm!: FormGroup;
  addressForm!: FormGroup;

  ngOnInit(): void {
    this.buildProfileForm();
    this.buildAddressForm();
    this.store.loadUser().then(() => {
      const user = this.store.user();
      if (user) {
        this.profileForm.patchValue({
          name: user.name,
          surname: user.surname,
          email: user.email,
          phone_number: user.phone_number ?? '',
        });
      }
    });
    this.store.loadAddresses();
  }

  private buildProfileForm(): void {
    this.profileForm = this.fb.group({
      name: ['', Validators.required],
      surname: ['', Validators.required],
      email: ['', [Validators.required, Validators.email]],
      phone_number: [''],
    });
  }

  private buildAddressForm(): void {
    this.addressForm = this.fb.group({
      street: ['', Validators.required],
      building_num: ['', Validators.required],
      floor_num: [null],
      apartment_num: [''],
      post_index: ['', Validators.required],
    });
  }

  async saveProfile(): Promise<void> {
    if (this.profileForm.invalid) return;

    const fv = this.profileForm.getRawValue();
    const ok = await this.store.updateUser({
      name: fv.name,
      surname: fv.surname,
      email: fv.email,
      phone_number: fv.phone_number || undefined,
    });

    if (ok) {
      this.profileForm.markAsPristine();
      this.messageService.add({
        severity: 'success',
        summary: 'profile.update.success',
        detail: 'profile.update.successDetail',
        life: 3000,
      });
    } else {
      this.messageService.add({
        severity: 'error',
        summary: 'profile.update.error',
        detail: this.store.error() ?? 'profile.update.errorDetail',
      });
    }
  }

  openAddAddressModal(): void {
    this.editingAddress.set(null);
    this.addressForm.reset();
    this.addressModalVisible.set(true);
  }

  openEditAddressModal(address: AddressResponse): void {
    this.editingAddress.set(address);
    this.addressForm.patchValue({
      street: address.street,
      building_num: address.building_num,
      floor_num: address.floor_num,
      apartment_num: address.apartment_num ?? '',
      post_index: address.post_index,
    });
    this.addressModalVisible.set(true);
  }

  closeAddressModal(): void {
    this.addressModalVisible.set(false);
    this.editingAddress.set(null);
  }

  onAddressModalVisibleChange(visible: boolean): void {
    if (!visible) this.closeAddressModal();
  }

  async saveAddress(): Promise<void> {
    this.addressForm.markAllAsTouched();
    if (this.addressForm.invalid) return;

    const fv = this.addressForm.getRawValue();
    const request = {
      street: fv.street,
      building_num: fv.building_num,
      floor_num: fv.floor_num || undefined,
      apartment_num: fv.apartment_num || undefined,
      post_index: fv.post_index,
    };

    let ok: boolean;
    if (this.editingAddress()) {
      ok = await this.store.updateAddress(this.editingAddress()!.id, request);
      if (ok) {
        this.messageService.add({ severity: 'success', summary: 'Updated', detail: 'Address updated', life: 3000 });
      } else {
        this.messageService.add({ severity: 'error', summary: 'Error', detail: this.store.error() ?? 'Could not update address' });
      }
    } else {
      ok = await this.store.addAddress(request);
      if (ok) {
        this.messageService.add({ severity: 'success', summary: 'Added', detail: 'Address added', life: 3000 });
      } else {
        this.messageService.add({ severity: 'error', summary: 'Error', detail: this.store.error() ?? 'Could not add address' });
      }
    }

    if (ok) {
      this.closeAddressModal();
    }
  }

  async deleteAddress(addressId: string): Promise<void> {
    const ok = await this.store.deleteAddress(addressId);
    if (ok) {
      this.messageService.add({ severity: 'success', summary: 'Deleted', detail: 'Address deleted', life: 3000 });
    } else {
      this.messageService.add({ severity: 'error', summary: 'Error', detail: this.store.error() ?? 'Could not delete address' });
    }
  }
}
