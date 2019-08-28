import { Component, OnInit } from '@angular/core';
import { Order } from '../api.service';

@Component({
  selector: 'frontend-orders',
  templateUrl: './orders.component.html',
  styleUrls: ['./orders.component.scss']
})
export class OrdersComponent implements OnInit {
  data: Order[] = [
    {
      order_id: "1",
      amount: "1000",
      status: "PendingPayment",
      buyer_address: "0xea674fdde714fd979de3edf0f56aa9716b898ec8",
      payment_transaction_id: "0xea674fdde714fd979de3edf0f56aa9716b898ec8",
      settlement_transaction_id: "0xea674fdde714fd979de3edf0f56aa9716b898ec8",
    },
    {
      order_id: "2",
      amount: "5000",
      status: "PendingResponse",
      buyer_address: "0xea674fdde714fd979de3edf0f56aa9716b898ec8",
      payment_transaction_id: "0xea674fdde714fd979de3edf0f56aa9716b898ec8",
      settlement_transaction_id: "0xea674fdde714fd979de3edf0f56aa9716b898ec8",
    },
    {
      order_id: "3",
      amount: "5000",
      status: "Delivering",
      buyer_address: "0xea674fdde714fd979de3edf0f56aa9716b898ec8",
      payment_transaction_id: "0xea674fdde714fd979de3edf0f56aa9716b898ec8",
      settlement_transaction_id: "0xea674fdde714fd979de3edf0f56aa9716b898ec8",
    }
  ];

  constructor() { }

  ngOnInit() {
  }

}
