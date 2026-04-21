import { Component, EventEmitter, Input, Output } from '@angular/core';
import { PaginatorModule, PaginatorState } from 'primeng/paginator';

@Component({
  selector: 'app-pagination',
  standalone: true,
  imports: [PaginatorModule],
  templateUrl: './pagination.component.html',
  styleUrl: './pagination.component.scss',
})
export class PaginationComponent {
  @Input() page = 1;
  @Input() perPage = 20;
  @Input() total = 0;
  @Output() pageChange = new EventEmitter<{ page: number; perPage: number }>();

  onPageChange(event: PaginatorState): void {
    const page = event.page !== undefined ? event.page + 1 : 1;
    const perPage = event.rows ?? this.perPage;
    this.pageChange.emit({ page, perPage });
  }
}
