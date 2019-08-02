import { Component, OnInit } from '@angular/core';

import { LocalDataSource } from 'ng2-smart-table';
import { EscrowMerchantService } from '../escrow-merchant.service';
@Component({
  selector: 'frontend-order-list',
  templateUrl: './order-list.component.html',
  styleUrls: ['./order-list.component.css']
})
export class OrderListComponent implements OnInit {
  source: LocalDataSource;

  settings = {
    orderId: {
      field: 'Order ID',
      filter: false,
      sort: true,
    },
    amount: {
      field: 'Amount',
      filter: false,
    },
    status: {
      field: 'Status',
      filter: false,
    },
  }

  // settings = {
  //   columns: {
  //     id: {
  //       title: 'ID',
  //       filter: false,
  //     },
  //     name: {
  //       title: 'Full Name',
  //       filter: {
  //         type: 'list',
  //         config: {
  //           selectText: 'Select...',
  //           list: [
  //             { value: 'Glenna Reichert', title: 'Glenna Reichert' },
  //             { value: 'Kurtis Weissnat', title: 'Kurtis Weissnat' },
  //             { value: 'Chelsey Dietrich', title: 'Chelsey Dietrich' },
  //           ],
  //         },
  //       },
  //     },
  //     email: {
  //       title: 'Email',
  //       filter: {
  //         type: 'completer',
  //         config: {
  //           completer: {
  //             data: this.data,
  //             searchFields: 'email',
  //             titleField: 'email',
  //           },
  //         },
  //       },
  //     },
  //     passed: {
  //       title: 'Passed',
  //       filter: {
  //         type: 'checkbox',
  //         config: {
  //           true: 'Yes',
  //           false: 'No',
  //           resetText: 'clear',
  //         },
  //       },
  //     },
  //   },
  // };

  constructor(private escrowMerchantService: EscrowMerchantService) {
    // this.source = new LocalDataSource(this.data);
  }

  ngOnInit() {
    console.log(this.escrowMerchantService);
    this.escrowMerchantService.getOrderList().subscribe(console.log);
  }

  onSearch(query: string = '') {
    // this.source.load(this.newData);
    this.source.setFilter([
      {
        field: 'orderId',
        search: query,
      },
    ], false);
    // second parameter specifying whether to perform 'AND' or 'OR' search
    // (meaning all columns should contain search query or at least one)
    // 'AND' by default, so changing to 'OR' by setting false here
  }
}
