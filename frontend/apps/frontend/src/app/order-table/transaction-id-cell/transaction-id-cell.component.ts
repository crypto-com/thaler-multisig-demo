import { Component, OnInit, Input } from '@angular/core';
import { ViewCell } from 'ng2-smart-table';

@Component({
  selector: 'frontend-transaction-id-cell',
  templateUrl: './transaction-id-cell.component.html',
  styleUrls: ['./transaction-id-cell.component.scss']
})
export class TransactionIdCellComponent implements ViewCell, OnInit {
  @Input() value: string;
  @Input() rowData: any;

  constructor() { }

  ngOnInit() {
  }

}
