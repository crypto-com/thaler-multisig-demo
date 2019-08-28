import { Component, OnInit, Input, Output, EventEmitter } from "@angular/core";

import { NgForm } from "@angular/forms";
import { OrderStatus, ApiService } from '../api.service';

@Component({
  selector: "app-mark-order-form",
  templateUrl: "./mark-order-form.component.html",
  styleUrls: ["./mark-order-form.component.scss"]
})
export class MarkOrderFormComponent implements OnInit {
  @Input() partyA: string;
  @Input() partyAAmount: string;
  @Input() partyB: string;
  @Input() partyBAmount: string;
  @Input() status: OrderStatus;
  @Output() submitted = new EventEmitter<void>();
  @Output() cancelled = new EventEmitter<void>();

  constructor() { }

  ngOnInit() { }

  handleSubmit(form: NgForm): void {
    this.markFormAsDirty(form);
    this.submitted.emit();
  }

  markFormAsDirty(form: NgForm) {
    Object.keys(form.controls).forEach(field => {
      form.controls[field].markAsDirty();
    });
  }

  cancel(): void {
    this.cancelled.emit();
  }
}
