import { Component, OnInit } from '@angular/core';
import { Order, ApiService } from '../api.service';
import { Router, NavigationEnd } from '@angular/router';
import { CurrentOrdersService } from './current-orders.service';
import { first } from 'rxjs/operators';
import { BehaviorSubject } from 'rxjs';

@Component({
  selector: 'frontend-orders',
  templateUrl: './orders.component.html',
  styleUrls: ['./orders.component.scss']
})
export class OrdersComponent implements OnInit {
  currentPath: string = '';

  constructor(
    private router: Router,
    private apiService: ApiService,
    private currentOrders: CurrentOrdersService
  ) {}

  ngOnInit() {
    this.currentPath = this.router.url;
    this.router.events.subscribe(event => {
      if (event instanceof NavigationEnd) {
        if (this.currentPath !== event.url) {
          this.currentPath = event.url;
          this.loadOrders();
        }
      }
    });

    this.subscribeOrders();
  }

  private subscribeOrders() {
    this.apiService.$pendingOrders.subscribe(orders => {
      if (this.currentPath === '/orders/pending') {
        this.currentOrders.$orders.next(orders);
      }
    });
    this.apiService.$outstandingOrders.subscribe(orders => {
      if (this.currentPath === '/orders/outstanding') {
        this.currentOrders.$orders.next(orders);
      }
    });
    this.apiService.$completedOrders.subscribe(orders => {
      if (this.currentPath === '/orders/completed') {
        this.currentOrders.$orders.next(orders);
      }
    });
  }

  private loadOrders() {
    let eventStream: BehaviorSubject<Order[]>;
    switch (this.currentPath) {
      case '/orders/pending':
        eventStream = this.apiService.$pendingOrders;
        break;
      case '/orders/outstanding':
        eventStream = this.apiService.$outstandingOrders;
        break;
      case '/orders/completed':
        eventStream = this.apiService.$completedOrders;
        break;
    }

    eventStream.pipe(first()).subscribe(orders => {
      this.currentOrders.$orders.next(orders);
    });
  }
}
