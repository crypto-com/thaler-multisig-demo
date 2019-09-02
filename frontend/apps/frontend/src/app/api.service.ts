import { Injectable } from '@angular/core';
import { HttpClient, HttpHeaders, HttpParams } from '@angular/common/http';
import { Observable, BehaviorSubject, interval } from 'rxjs';

@Injectable({
  providedIn: 'root'
})
export class ApiService {
  public $pendingOrders: BehaviorSubject<Order[]>;
  public $outstandingOrders: BehaviorSubject<Order[]>;
  public $completedOrders: BehaviorSubject<Order[]>;
  private baseUrl = 'http://localhost:8080';

  constructor(private http: HttpClient) {
    this.$pendingOrders = new BehaviorSubject([]);
    this.$outstandingOrders = new BehaviorSubject([]);
    this.$completedOrders = new BehaviorSubject([]);

    this.updateOrdersRegularly();
  }

  private updateOrdersRegularly() {
    this.updateOrders();
    interval(2000).subscribe(() => {
      this.updateOrders();
    });
  }

  updateOrders() {
    this.getPendingOrders().subscribe(orders => {
      this.$pendingOrders.next(orders);
    }, console.error);
    this.getOutstandingOrders().subscribe(orders => {
      this.$outstandingOrders.next(orders);
    }, console.error);
    this.getCompletedOrders().subscribe(orders => {
      this.$completedOrders.next(orders);
    }, console.error);
  }

  getPendingOrders(): Observable<Order[]> {
    const url = `${this.baseUrl}/order/pending`;
    return this.http.get<Order[]>(url);
  }

  getOutstandingOrders(): Observable<Order[]> {
    const url = `${this.baseUrl}/order/outstanding`;
    return this.http.get<Order[]>(url);
  }

  getCompletedOrders(): Observable<Order[]> {
    const url = `${this.baseUrl}/order/completed`;
    return this.http.get<Order[]>(url);
  }

  markDelivering(orderId: string): Observable<void> {
    const url = `${this.baseUrl}/order/delivering`;
    return this.http.post<void>(
      url,
      new HttpParams({
        fromObject: {
          order_id: orderId
        }
      }),
      {
        headers: new HttpHeaders({
          'Content-Type': 'application/x-www-form-urlencoded'
        })
      }
    );
  }

  markRefunding(orderId: string): Observable<void> {
    const url = `${this.baseUrl}/order/refunding`;
    return this.http.post<void>(
      url,
      new HttpParams({
        fromObject: {
          order_id: orderId
        }
      }),
      {
        headers: new HttpHeaders({
          'Content-Type': 'application/x-www-form-urlencoded'
        })
      }
    );
  }
}

export type Order = {
  order_id: string;
  amount: string;
  status: string;
  buyer_address: string;
  payment_transaction_id: string;
  settlement_transaction_id: string;
};

export enum OrderStatus {
  PendingPayment = 'PendingPayment',
  PendingResponse = 'PendingResponse',
  Delivering = 'Delivering',
  Refunding = 'Refunding',
  Completed = 'Completed',
  Refunded = 'Refunded'
}
