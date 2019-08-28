import { Component, OnInit } from '@angular/core';
import { Router, NavigationEnd } from '@angular/router';
import { ApiService } from '../api.service';

@Component({
  selector: 'frontend-side-navmenu',
  templateUrl: './side-navmenu.component.html',
  styleUrls: ['./side-navmenu.component.scss']
})
export class SideNavmenuComponent implements OnInit {
  currentRoutePath: string = "";
  outstandingOrderSize: number = 0;

  constructor(
    private router: Router,
    private apiService: ApiService,
  ) { }

  ngOnInit() {
    this.router.events.subscribe(event => {
      if (event instanceof NavigationEnd) {
        this.currentRoutePath = event.url;
      }
    });

    this.apiService.pendingResponseOrders.subscribe(orders => {
      this.outstandingOrderSize = orders.length;
    });
  }

}
