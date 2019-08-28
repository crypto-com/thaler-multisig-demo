import { Component, OnInit, Input, SimpleChanges } from '@angular/core';

import { LocalDataSource } from 'ng2-smart-table';
import { Order } from '../api.service';
import { StatusCellComponent } from './status-cell/status-cell.component';
import { TransactionIdCellComponent } from './transaction-id-cell/transaction-id-cell.component';
import { CurrentOrdersService } from '../orders/current-orders.service';
import { ActionCellComponent } from './action-cell/action-cell.component';
@Component({
  selector: 'frontend-order-table',
  templateUrl: './order-table.component.html',
  styleUrls: ['./order-table.component.scss']
})
export class OrderTableComponent implements OnInit {
  @Input() data: Order[];
  source: LocalDataSource;

  settings = {
    actions: false,
    edit: false,
    add: false,
    delete: false,
    hideSubHeader: true,
    columns: {
      order_id: {
        title: 'Order ID',
        filter: false,
        sort: true
      },
      amount: {
        title: 'Amount',
        filter: false,
        type: 'html',
        valuePrepareFunction: (value, row) => {
          return `$${value}.00`;
        },
      },
      currency: {
        title: 'Currency',
        filter: false,
        valuePrepareFunction: (value, row) => {
          return 'CRO';
        },
      },
      status: {
        title: 'Status',
        filter: {
          type: 'list',
          config: {
            selectText: 'Select status',
            list: [
              { value: 'PendingPayment', title: 'Pending' },
              { value: 'PendingResponse', title: 'Outstanding' },
              { value: 'Delivering', title: 'Delivering' },
              { value: 'Refunding', title: 'Refunding' },
              { value: 'Completed', title: 'Completed' },
              { value: 'Refunded', title: 'Refunded' },
            ],
          },
        },
        type: 'custom',
        renderComponent: StatusCellComponent
      },
      buyer_address: {
        title: 'Buyer Address',
        filter: true,
        type: 'custom',
        renderComponent: TransactionIdCellComponent
      },
      payment_transaction_id: {
        title: 'Payment Transaction',
        filter: false,
        type: 'custom',
        renderComponent: TransactionIdCellComponent
      },
      settlement_transaction_id: {
        title: 'Settlement Transaction',
        filter: false,
        type: 'custom',
        renderComponent: TransactionIdCellComponent
      },
      action: {
        title: 'Action',
        type: 'custom',
        renderComponent: ActionCellComponent
      }
    }
  };

  constructor(private currentOrders: CurrentOrdersService) {
    this.source = new LocalDataSource([]);
  }

  ngOnInit() {
    this.subscribeToOrders();
  }

  private subscribeToOrders() {
    this.currentOrders.$orders.subscribe(orders => {
      console.log(orders);
      this.source.load(orders);
    })
  }

  onSearch(query: string = '') {
    if (query === '') {
      this.source.setFilter([]);
      return;
    }
    this.source.setFilter(
      [
        {
          field: 'order_id',
          search: query
        },
        {
          field: 'status',
          search: query
        },
        {
          field: 'amount',
          search: query
        },
        {
          field: 'payment_transaction_id',
          search: query
        },
        {
          field: 'settlement_transaction_id',
          search: query
        }
      ],
      false
    );
    // second parameter specifying whether to perform 'AND' or 'OR' search
    // (meaning all columns should contain search query or at least one)
    // 'AND' by default, so changing to 'OR' by setting false here
  }
}
