import { Injectable } from '@angular/core';
import { BehaviorSubject } from 'rxjs';
import { Order } from '../api.service';

@Injectable({
  providedIn: 'root'
})
export class CurrentOrdersService {
  $orders: BehaviorSubject<Order[]>;

  constructor() {
    this.$orders = new BehaviorSubject([]);
  }
}
