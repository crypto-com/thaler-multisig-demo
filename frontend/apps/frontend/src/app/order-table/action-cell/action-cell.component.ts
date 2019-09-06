import { Component, OnInit, Input, TemplateRef } from '@angular/core';
import { BsModalRef, BsModalService } from 'ngx-bootstrap/modal';
import BigNumber from 'bignumber.js';
import { ApiService } from '../../api.service';

@Component({
  selector: 'frontend-action-cell',
  templateUrl: './action-cell.component.html',
  styleUrls: ['./action-cell.component.scss']
})
export class ActionCellComponent implements OnInit {
  @Input() value: string;
  @Input() rowData: any;

  modalRef: BsModalRef;
  modalConfig = {
    backdrop: true,
    ignoreBackdropClick: true
  };

  amount: string;
  orderAmount: string;

  constructor(
    private modalService: BsModalService,
    private apiService: ApiService
  ) {}

  ngOnInit() {
    this.amount = this.rowData.amount;
    this.orderAmount = new BigNumber(this.rowData.amount).minus(10).toString();
  }

  openModal(template: TemplateRef<any>) {
    this.modalRef = this.modalService.show(template, this.modalConfig);
  }

  closeModal() {
    this.modalRef.hide();
  }

  markDelivered() {
    // TODO: Error handling
    this.apiService
      .markDelivering(this.rowData.order_id)
      .subscribe(() => {
        this.apiService.updateOrders();
      }, console.error);
    this.closeModal();
  }

  markRefund() {
    // TODO: Error handling
    this.apiService
      .markRefunding(this.rowData.order_id)
      .subscribe(() => {
        this.apiService.updateOrders();
      }, console.error);
    this.closeModal();
  }
}
