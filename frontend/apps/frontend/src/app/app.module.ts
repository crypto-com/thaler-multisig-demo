import { BrowserModule } from '@angular/platform-browser';
import { HttpClientModule, HttpClient } from '@angular/common/http';
import { NgModule } from '@angular/core';
import { TranslateModule, TranslateLoader } from '@ngx-translate/core';
import { TranslateHttpLoader } from '@ngx-translate/http-loader';
import { Ng2SmartTableModule } from 'ng2-smart-table';
import { AngularFontAwesomeModule } from 'angular-font-awesome';
import { BrowserAnimationsModule } from '@angular/platform-browser/animations';

import { MatSidenavModule } from '@angular/material/sidenav';
import { MatIconModule } from '@angular/material/icon';
import { MatBadgeModule } from '@angular/material/badge';
import { MatChipsModule } from '@angular/material/chips';
import { MatTabsModule } from '@angular/material/tabs';

import { AppComponent } from './app.component';
import { AppRoutingModule } from './app-routing.module';
import { OrderTableComponent } from './order-table/order-table.component';
import { SideNavmenuComponent } from './side-navmenu/side-navmenu.component';
import { OrdersComponent } from './orders/orders.component';
import { DashboardComponent } from './dashboard/dashboard.component';
import { StatusCellComponent } from './order-table/status-cell/status-cell.component';
import { TransactionIdCellComponent } from './order-table/transaction-id-cell/transaction-id-cell.component';
import { OrderTableactionCellComponent } from './order-tableaction-cell/order-tableaction-cell.component';

export function HttpLoaderFactory(http: HttpClient) {
  return new TranslateHttpLoader(http);
}

@NgModule({
  entryComponents: [
    StatusCellComponent,
    TransactionIdCellComponent
  ],
  declarations: [
    AppComponent,
    OrderTableComponent,
    SideNavmenuComponent,
    OrdersComponent,
    DashboardComponent,
    StatusCellComponent,
    TransactionIdCellComponent,
    OrderTableactionCellComponent
  ],
  imports: [
    BrowserModule,
    HttpClientModule,
    TranslateModule.forRoot({
      loader: {
        provide: TranslateLoader,
        useFactory: HttpLoaderFactory,
        deps: [HttpClient]
      }
    }),
    AppRoutingModule,
    Ng2SmartTableModule,
    AngularFontAwesomeModule,
    BrowserAnimationsModule,
    MatSidenavModule,
    MatIconModule,
    MatBadgeModule,
    MatChipsModule,
    MatTabsModule
  ],
  providers: [],
  bootstrap: [AppComponent]
})
export class AppModule {}
