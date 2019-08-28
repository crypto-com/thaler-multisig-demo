import { Component, OnInit, Input } from '@angular/core';
import { OrderStatus } from '../../api.service';
import { ViewCell } from 'ng2-smart-table';

@Component({
  selector: 'frontend-status-cell',
  templateUrl: './status-cell.component.html',
  styleUrls: ['./status-cell.component.scss']
})
export class StatusCellComponent implements ViewCell, OnInit {
  @Input() value: string;
  @Input() rowData: any;

  renderValue: string;
  styleClass: string;

  constructor() { }

  ngOnInit() {
    this.renderValue = this.renderText(this.value);
    this.styleClass = this.renderStyle(this.value);
  }

  renderText(status: string): string {
    switch (status) {
      case OrderStatus.PendingPayment:
        return "Pending";
      case OrderStatus.PendingResponse:
        return "Outstanding";
      case OrderStatus.Delivering:
        return "Delivering";
      case OrderStatus.Refunding:
        return "Refunding";
      case OrderStatus.Completed:
        return "Completed";
      case OrderStatus.Refunded:
        return "Refunded";
    }
  }

  renderStyle(status: string): string {
    switch (status) {
      case OrderStatus.PendingPayment:
        return "secondary";
      case OrderStatus.PendingResponse:
        return "warning";
      case OrderStatus.Delivering:
        return "info";
      case OrderStatus.Refunding:
        return "danger";
      case OrderStatus.Completed:
        return "success";
      case OrderStatus.Refunded:
        return "danger";
    }

  }
}
