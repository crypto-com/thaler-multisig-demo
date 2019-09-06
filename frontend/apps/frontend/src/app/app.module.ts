import { BrowserModule } from '@angular/platform-browser';
import { HttpClientModule, HttpClient } from '@angular/common/http';
import { NgModule } from '@angular/core';
import { TranslateModule, TranslateLoader } from '@ngx-translate/core';
import { TranslateHttpLoader } from '@ngx-translate/http-loader';
import { Ng2SmartTableModule } from 'ng2-smart-table';
import { AngularFontAwesomeModule } from 'angular-font-awesome';
import { BrowserAnimationsModule } from '@angular/platform-browser/animations';

import { ModalModule } from "ngx-bootstrap/modal";
import { MatSidenavModule } from '@angular/material/sidenav';
import { MatIconModule } from '@angular/material/icon';
import { MatBadgeModule } from '@angular/material/badge';
import { MatChipsModule } from '@angular/material/chips';
import { MatTabsModule } from '@angular/material/tabs';
import { MatButtonModule } from '@angular/material/button';

import { AppComponent } from './app.component';
import { AppRoutingModule } from './app-routing.module';
import { OrderTableComponent } from './order-table/order-table.component';
import { SideNavmenuComponent } from './side-navmenu/side-navmenu.component';
import { OrdersComponent } from './orders/orders.component';
import { DashboardComponent } from './dashboard/dashboard.component';
import { StatusCellComponent } from './order-table/status-cell/status-cell.component';
import { TransactionIdCellComponent } from './order-table/transaction-id-cell/transaction-id-cell.component';
import { OrderTableactionCellComponent } from './order-tableaction-cell/order-tableaction-cell.component';
import { ActionCellComponent } from './order-table/action-cell/action-cell.component';
import { MarkOrderFormComponent } from './mark-order-form/mark-order-form.component';
import { FormsModule } from '@angular/forms';

export function HttpLoaderFactory(http: HttpClient) {
  return new TranslateHttpLoader(http);
}

@NgModule({
  entryComponents: [
    StatusCellComponent,
    TransactionIdCellComponent,
    ActionCellComponent
  ],
  declarations: [
    AppComponent,
    OrderTableComponent,
    SideNavmenuComponent,
    OrdersComponent,
    DashboardComponent,
    StatusCellComponent,
    TransactionIdCellComponent,
    OrderTableactionCellComponent,
    ActionCellComponent,
    MarkOrderFormComponent
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
    FormsModule,
    ModalModule.forRoot(),
    MatSidenavModule,
    MatIconModule,
    MatBadgeModule,
    MatChipsModule,
    MatTabsModule,
    MatButtonModule
  ],
  providers: [],
  bootstrap: [AppComponent]
})
export class AppModule {}
