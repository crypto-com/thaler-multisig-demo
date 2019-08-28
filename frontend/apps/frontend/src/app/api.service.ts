import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable, BehaviorSubject, interval } from 'rxjs'

@Injectable({
  providedIn: 'root'
})
export class ApiService {
  public pendingPaymentOrders: BehaviorSubject<Order[]>;
  public pendingResponseOrders: BehaviorSubject<Order[]>;
  public settledOrders: BehaviorSubject<Order[]>;
  private baseUrl = 'http://localhost:8080';

  constructor(private http: HttpClient) {
    this.pendingPaymentOrders = new BehaviorSubject([]);
    this.pendingResponseOrders = new BehaviorSubject([]);
    this.settledOrders = new BehaviorSubject([]);

    this.updateRegularly();
  }

  private updateRegularly() {
    interval(5000).subscribe(() => {
      this.update();
    });
  }

  private update() {
    this.getPendingPaymentOrders().subscribe(orders => {
      this.pendingPaymentOrders.next(orders);
    }, console.error);
    this.getPendingResponseOrders().subscribe(orders => {
      this.pendingResponseOrders.next(orders);
    }, console.error);
    this.getSettledOrders().subscribe(orders => {
      this.settledOrders.next(orders);
    }, console.error);
  }

  getPendingPaymentOrders(): Observable<Order[]> {
    const url = `${this.baseUrl}/order/pending`;
    return this.http.get<Order[]>(url);
  }

  getPendingResponseOrders(): Observable<Order[]> {
    const url = `${this.baseUrl}/order/outstanding`;
    return this.http.get<Order[]>(url);
  }

  getSettledOrders(): Observable<Order[]> {
    const url = `${this.baseUrl}/order/completed`;
    return this.http.get<Order[]>(url);
  }
}

export type Order = {
  order_id: string;
  amount: string;
  status: string;
  buyer_address: string;
  payment_transaction_id: string;
  settlement_transaction_id: string;
}

export enum OrderStatus {
  PendingPayment = 'PendingPayment',
  PendingResponse = 'PendingResponse',
  Delivering = 'Delivering',
  Refunding = 'Refunding',
  Delivered = 'Delivered',
  Refunded = 'Refunded',
}
