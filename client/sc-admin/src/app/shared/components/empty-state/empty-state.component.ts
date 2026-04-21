import { Component, ChangeDetectionStrategy, input } from '@angular/core';
import { TranslateModule } from '@ngx-translate/core';

@Component({
  selector: 'app-empty-state',
  standalone: true,
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [TranslateModule],
  templateUrl: './empty-state.component.html',
  styleUrl: './empty-state.component.scss',
})
export class EmptyStateComponent {
  icon = input<string>('pi pi-inbox');
  titleKey = input<string>('emptyState.default');
  messageKey = input<string>('');
}
